#![allow(clippy::cast_precision_loss, clippy::cast_lossless)]

use super::CandidateGenerator;
use crate::{Candidate, Evidence, RuleContext, StandardLexicon, TextRange};

#[derive(Clone, Debug)]
pub struct SpacingCandidateGenerator {
    lexicon: StandardLexicon,
    max_candidates: usize,
}

impl SpacingCandidateGenerator {
    #[must_use]
    pub fn new(lexicon: StandardLexicon, max_candidates: usize) -> Self {
        Self {
            lexicon,
            max_candidates: max_candidates.max(1),
        }
    }
}

impl CandidateGenerator for SpacingCandidateGenerator {
    fn generate(&self, context: &RuleContext<'_>) -> Vec<Candidate> {
        let words = context.document().words();
        let mut candidates = Vec::new();
        for word in words {
            if self.lexicon.lookup(&word.surface).is_some() {
                continue;
            }
            let characters: Vec<_> = word.surface.char_indices().collect();
            for &(split_offset, _) in characters.iter().skip(1) {
                let left = &word.surface[..split_offset];
                let right = &word.surface[split_offset..];
                let Some(left_entry) = self.lexicon.lookup(left) else {
                    continue;
                };
                let Some(right_entry) = self.lexicon.lookup(right) else {
                    continue;
                };
                let score = ((left_entry.frequency as f32 + right_entry.frequency as f32 + 2.0)
                    .ln()
                    / 32.0)
                    .min(1.0);
                candidates.push(
                    Candidate::new(
                        "spacing.oov.split",
                        word.range,
                        &word.surface,
                        format!("{left} {right}"),
                    )
                    .with_evidence(Evidence::new(
                        "lexicon-split",
                        format!("{left}+{right}"),
                        score as f64,
                    ))
                    .with_score(score),
                );
            }
        }

        for pair in words.windows(2) {
            let gap = &context.text()[pair[0].range.end..pair[1].range.start];
            if !gap.chars().all(char::is_whitespace) {
                continue;
            }
            let joined = format!("{}{}", pair[0].surface, pair[1].surface);
            let Some(entry) = self.lexicon.lookup(&joined) else {
                continue;
            };
            let range = TextRange {
                start: pair[0].range.start,
                end: pair[1].range.end,
            };
            let score = ((entry.frequency as f32 + 1.0).ln() / 16.0).min(1.0);
            candidates.push(
                Candidate::new(
                    "spacing.oov.join",
                    range,
                    &context.text()[range.start..range.end],
                    joined,
                )
                .with_evidence(Evidence::new(
                    "lexicon-join",
                    entry.part_of_speech.clone(),
                    score as f64,
                ))
                .with_score(score),
            );
        }

        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.range.start.cmp(&right.range.start))
        });
        candidates.truncate(self.max_candidates);
        candidates
    }
}
