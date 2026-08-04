use crate::{Candidate, RuleContext};

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
