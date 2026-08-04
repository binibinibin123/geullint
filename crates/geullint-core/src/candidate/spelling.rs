#![allow(clippy::cast_precision_loss, clippy::cast_lossless)]

use super::CandidateGenerator;
use crate::{Candidate, Evidence, RuleContext, StandardLexicon, TextRange, phonology_distance};

#[derive(Clone, Debug)]
pub struct SpellingCandidateGenerator {
    lexicon: StandardLexicon,
    max_candidates: usize,
}

impl SpellingCandidateGenerator {
    #[must_use]
    pub fn new(lexicon: StandardLexicon, max_candidates: usize) -> Self {
        Self {
            lexicon,
            max_candidates: max_candidates.max(1),
        }
    }

    fn candidates_for_word(&self, surface: &str, range: TextRange) -> Vec<Candidate> {
        let source_characters: Vec<_> = surface.chars().collect();
        if source_characters.len() < 2 || self.lexicon.lookup(surface).is_some() {
            return Vec::new();
        }
        let mut candidates = Vec::new();
        for entry in self.lexicon.entries() {
            let target_characters: Vec<_> = entry.surface.chars().collect();
            if target_characters.len().abs_diff(source_characters.len()) > 1 {
                continue;
            }
            let edit_distance = levenshtein(&source_characters, &target_characters);
            if edit_distance > 2 {
                continue;
            }
            let phonology = source_characters
                .iter()
                .zip(target_characters.iter())
                .map(|(left, right)| phonology_distance(*left, *right))
                .sum::<u8>();
            let frequency = (entry.frequency as f32 + 1.0).ln();
            let score = (1.0 / (1.0 + edit_distance as f32))
                + (1.0 / (1.0 + phonology as f32)) * 0.35
                + frequency.min(16.0) / 64.0;
            candidates.push(
                Candidate::new("spelling.oov.near", range, surface, &entry.surface)
                    .with_evidence(Evidence::new(
                        "edit-distance",
                        edit_distance.to_string(),
                        score as f64,
                    ))
                    .with_evidence(Evidence::new(
                        "frequency",
                        entry.frequency.to_string(),
                        frequency as f64,
                    ))
                    .with_score(score),
            );
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.replacement.cmp(&right.replacement))
        });
        candidates.truncate(self.max_candidates);
        candidates
    }
}

impl CandidateGenerator for SpellingCandidateGenerator {
    fn generate(&self, context: &RuleContext<'_>) -> Vec<Candidate> {
        let mut candidates = context
            .document()
            .words()
            .iter()
            .flat_map(|word| self.candidates_for_word(&word.surface, word.range))
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.range.start.cmp(&right.range.start))
                .then_with(|| left.range.end.cmp(&right.range.end))
                .then_with(|| left.replacement.cmp(&right.replacement))
        });
        candidates.truncate(self.max_candidates);
        candidates
    }
}

fn levenshtein(left: &[char], right: &[char]) -> usize {
    let mut previous: Vec<_> = (0..=right.len()).collect();
    for (left_index, left_character) in left.iter().enumerate() {
        let mut current = vec![left_index + 1; right.len() + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            current[right_index + 1] = if left_character == right_character {
                previous[right_index]
            } else {
                1 + previous[right_index]
                    .min(previous[right_index + 1])
                    .min(current[right_index])
            };
        }
        previous = current;
    }
    previous[right.len()]
}
