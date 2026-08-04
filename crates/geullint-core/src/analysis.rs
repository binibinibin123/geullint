use crate::{MorphToken, SourceKind, TextRange, is_hangul_syllable, source_ranges};
use serde::Serialize;

#[path = "analysis/lattice.rs"]
pub mod lattice;

#[cfg(feature = "morphology")]
use crate::MorphAnalyzer;

/// One Korean word that is safe for lint rules to inspect.
///
/// The range always points into the original UTF-8 source. Words in Markdown code
/// spans and non-comment source code are deliberately excluded with the same source
/// selection policy as the rest of the linter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzedWord {
    pub surface: String,
    pub range: TextRange,
    pub part_of_speech: Option<String>,
}

/// A source-safe, reusable view of Korean words in one document.
///
/// Builds with the opt-in `morphology` feature enrich words with the bundled local
/// dictionary whenever one morphology token covers the full word. Compact builds
/// retain the same word boundaries without downloading a dictionary or sending text
/// off-device.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AnalyzedDocument {
    source_ranges: Vec<TextRange>,
    words: Vec<AnalyzedWord>,
    morphology_tokens: Vec<MorphToken>,
}

impl AnalyzedDocument {
    #[must_use]
    pub fn new(text: &str, source_kind: SourceKind) -> Self {
        let source_ranges = source_ranges(text, source_kind);
        let words = korean_words_in_ranges(text, &source_ranges);

        #[cfg(feature = "morphology")]
        let (words, morphology_tokens) = {
            let mut words = words;
            let morphology_tokens = analyze_morphology(text, &source_ranges);
            enrich_words_from_sorted_tokens(&morphology_tokens, &mut words);
            (words, morphology_tokens)
        };

        #[cfg(not(feature = "morphology"))]
        let morphology_tokens = Vec::new();

        Self {
            source_ranges,
            words,
            morphology_tokens,
        }
    }

    #[must_use]
    pub fn source_ranges(&self) -> &[TextRange] {
        &self.source_ranges
    }

    #[must_use]
    pub fn words(&self) -> &[AnalyzedWord] {
        &self.words
    }

    /// Returns every dictionary token whose range is safe for lint rules to inspect.
    ///
    /// Compact builds without the `morphology` feature return an empty slice while
    /// preserving the same API and surface-based diagnostics.
    #[must_use]
    pub fn morphology_tokens(&self) -> &[MorphToken] {
        &self.morphology_tokens
    }
}

fn korean_words_in_ranges(text: &str, source_ranges: &[TextRange]) -> Vec<AnalyzedWord> {
    let mut words = Vec::new();

    for source_range in source_ranges {
        let source = &text[source_range.start..source_range.end];
        let mut word_start = None;

        for (relative_offset, character) in source.char_indices() {
            if is_hangul_syllable(character) {
                word_start.get_or_insert(relative_offset);
                continue;
            }

            if let Some(start) = word_start.take() {
                let range = TextRange {
                    start: source_range.start + start,
                    end: source_range.start + relative_offset,
                };
                words.push(AnalyzedWord {
                    surface: text[range.start..range.end].to_owned(),
                    range,
                    part_of_speech: None,
                });
            }
        }

        if let Some(start) = word_start {
            let range = TextRange {
                start: source_range.start + start,
                end: source_range.end,
            };
            words.push(AnalyzedWord {
                surface: text[range.start..range.end].to_owned(),
                range,
                part_of_speech: None,
            });
        }
    }

    words
}

#[cfg(feature = "morphology")]
thread_local! {
    static BUNDLED_MORPH_ANALYZER: std::cell::RefCell<Option<MorphAnalyzer>> = const {
        std::cell::RefCell::new(None)
    };
}

#[cfg(feature = "morphology")]
fn analyze_morphology(text: &str, source_ranges: &[TextRange]) -> Vec<MorphToken> {
    let tokens = BUNDLED_MORPH_ANALYZER.with(|analyzer| {
        let mut analyzer = analyzer.borrow_mut();
        if analyzer.is_none() {
            *analyzer = MorphAnalyzer::bundled().ok();
        }
        analyzer
            .as_ref()
            .and_then(|analyzer| analyzer.analyze(text).ok())
            .unwrap_or_default()
    });
    tokens_in_sorted_ranges(tokens, source_ranges)
}

#[cfg(any(feature = "morphology", test))]
fn tokens_in_sorted_ranges(
    tokens: Vec<MorphToken>,
    source_ranges: &[TextRange],
) -> Vec<MorphToken> {
    let mut range_index = 0;
    tokens
        .into_iter()
        .filter(|token| {
            while source_ranges
                .get(range_index)
                .is_some_and(|range| range.end <= token.range.start)
            {
                range_index += 1;
            }
            source_ranges.get(range_index).is_some_and(|range| {
                token.range.start >= range.start && token.range.end <= range.end
            })
        })
        .collect()
}

#[cfg(any(feature = "morphology", test))]
fn enrich_words_from_sorted_tokens(tokens: &[MorphToken], words: &mut [AnalyzedWord]) {
    let mut token_index = 0;
    for word in words {
        while tokens.get(token_index).is_some_and(|token| {
            (token.range.start, token.range.end) < (word.range.start, word.range.end)
        }) {
            token_index += 1;
        }
        if let Some(token) = tokens
            .get(token_index)
            .filter(|token| token.range == word.range)
        {
            word.part_of_speech = Some(token.part_of_speech.clone());
        }
    }
}

#[cfg(all(test, not(feature = "morphology")))]
mod tests {
    use super::*;

    #[test]
    fn compact_analysis_exposes_an_empty_morphology_token_slice() {
        let document = AnalyzedDocument::new("책을 읽는다.", SourceKind::PlainText);

        assert!(document.morphology_tokens().is_empty());
    }

    #[test]
    fn sorted_range_merge_keeps_only_tokens_fully_inside_a_source_range() {
        let ranges = [
            TextRange { start: 3, end: 9 },
            TextRange { start: 15, end: 21 },
        ];
        let tokens = vec![
            token("밖", 0, 3),
            token("안", 3, 6),
            token("경계", 6, 12),
            token("안쪽", 15, 21),
            token("뒤", 24, 27),
        ];

        let filtered = tokens_in_sorted_ranges(tokens, &ranges);

        assert_eq!(
            filtered
                .iter()
                .map(|token| (token.surface.as_str(), token.range))
                .collect::<Vec<_>>(),
            [
                ("안", TextRange { start: 3, end: 6 }),
                ("안쪽", TextRange { start: 15, end: 21 }),
            ]
        );
    }

    #[test]
    fn sorted_word_merge_enriches_only_an_exact_token_range() {
        let mut words = vec![
            word("첫말", 0, 6),
            word("온말", 7, 13),
            word("끝말", 14, 20),
        ];
        let tokens = vec![
            token("첫", 0, 3),
            token("말", 3, 6),
            token("온말", 7, 13),
            token("뒤", 21, 24),
        ];

        enrich_words_from_sorted_tokens(&tokens, &mut words);

        assert_eq!(words[0].part_of_speech, None);
        assert_eq!(words[1].part_of_speech.as_deref(), Some("NNG"));
        assert_eq!(words[2].part_of_speech, None);
    }

    fn token(surface: &str, start: usize, end: usize) -> MorphToken {
        MorphToken {
            surface: surface.to_owned(),
            part_of_speech: "NNG".to_owned(),
            range: TextRange { start, end },
        }
    }

    fn word(surface: &str, start: usize, end: usize) -> AnalyzedWord {
        AnalyzedWord {
            surface: surface.to_owned(),
            range: TextRange { start, end },
            part_of_speech: None,
        }
    }
}
