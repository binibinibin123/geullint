use crate::{Candidate, Confidence, RuleContext};

/// Scores a candidate using only local, deterministic features.
pub trait CandidateScorer {
    fn score(&self, candidate: &Candidate, context: &RuleContext<'_>) -> f32;
}

/// Baseline scorer used before a learned ranker is available. It makes the old rule order an
/// explicit, replaceable stage rather than an implicit side effect of the monolithic engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicScorer;

impl CandidateScorer for DeterministicScorer {
    fn score(&self, candidate: &Candidate, _context: &RuleContext<'_>) -> f32 {
        if candidate.original == candidate.replacement {
            0.0
        } else if candidate.replacement.is_empty() {
            0.25
        } else {
            0.5
        }
    }
}

/// Interpretable feature weights for the compact local ranker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RankWeights {
    pub bias: f32,
    pub edit_distance: f32,
    pub phonology_distance: f32,
    pub log_frequency: f32,
    pub base_score: f32,
}

/// A small deterministic ranker used as the runtime contract for the future INT8 model.
///
/// It deliberately has no network or runtime model-loading path. Training/export tooling can
/// replace these weights later while preserving the same feature names and confidence policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeulRankSmall {
    weights: RankWeights,
}

impl Default for GeulRankSmall {
    fn default() -> Self {
        Self {
            weights: RankWeights {
                bias: 4.0,
                edit_distance: -0.9,
                phonology_distance: -0.45,
                log_frequency: 0.18,
                base_score: 0.35,
            },
        }
    }
}

impl GeulRankSmall {
    #[must_use]
    pub const fn from_weights(weights: RankWeights) -> Self {
        Self { weights }
    }

    #[must_use]
    pub const fn weights(self) -> RankWeights {
        self.weights
    }

    /// Score a candidate into the calibrated [0, 1] confidence interval.
    #[must_use]
    pub fn score(&self, candidate: &Candidate, _context: &RuleContext<'_>) -> f32 {
        if candidate.evidence.is_empty() {
            return 0.5;
        }
        let edit_distance = evidence_number(candidate, "edit-distance");
        let phonology_distance = evidence_number(candidate, "phonology-distance");
        let frequency = evidence_number(candidate, "frequency");
        let log_frequency = (frequency + 1.0).ln();
        let base_score = candidate.score;
        let linear = self.weights.bias
            + self.weights.edit_distance * edit_distance
            + self.weights.phonology_distance * phonology_distance
            + self.weights.log_frequency * log_frequency
            + self.weights.base_score * base_score;
        1.0 / (1.0 + (-linear).exp())
    }

    pub fn rank(&self, candidates: &mut [Candidate], context: &RuleContext<'_>) {
        for candidate in candidates.iter_mut() {
            candidate.score = self.score(candidate, context);
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.range.start.cmp(&right.range.start))
                .then_with(|| left.range.end.cmp(&right.range.end))
                .then_with(|| left.replacement.cmp(&right.replacement))
        });
    }

    #[must_use]
    pub fn confidence(&self, score: f32) -> Confidence {
        if score >= 0.85 {
            Confidence::High
        } else if score >= 0.6 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }
}

impl CandidateScorer for GeulRankSmall {
    fn score(&self, candidate: &Candidate, context: &RuleContext<'_>) -> f32 {
        Self::score(self, candidate, context)
    }
}

fn evidence_number(candidate: &Candidate, code: &str) -> f32 {
    candidate
        .evidence
        .iter()
        .find(|evidence| evidence.code == code)
        .and_then(|evidence| evidence.value.parse::<f32>().ok())
        .unwrap_or(0.0)
}
