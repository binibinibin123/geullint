#![allow(clippy::cast_precision_loss)]

use serde::{Deserialize, Serialize};

/// Conservative style profiles used only to lower automation confidence.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleProfile {
    #[default]
    Plain,
    Formal,
    Technical,
    Code,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleContext {
    pub profile: StyleProfile,
    pub sentence_count: usize,
    pub average_sentence_length: f32,
}

impl StyleContext {
    #[must_use]
    pub fn detect(text: &str) -> Self {
        let profile = if looks_like_code(text) {
            StyleProfile::Code
        } else if contains_any(text, &["본 문서", "규정", "보고서", "공문", "의거하여"])
        {
            StyleProfile::Formal
        } else if contains_any(text, &["API", "JSON", "Rust", "함수", "코드"]) {
            StyleProfile::Technical
        } else {
            StyleProfile::Plain
        };
        let sentence_count = text
            .chars()
            .filter(|character| matches!(character, '.' | '?' | '!' | '。' | '？' | '！'))
            .count()
            .max(1);
        let average_sentence_length = text.chars().count() as f32 / sentence_count as f32;
        Self {
            profile,
            sentence_count,
            average_sentence_length,
        }
    }
}

fn looks_like_code(text: &str) -> bool {
    contains_any(
        text,
        &[
            "fn ",
            "function ",
            "import ",
            "const ",
            "=>",
            "println!(",
            "{}",
        ],
    )
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}
