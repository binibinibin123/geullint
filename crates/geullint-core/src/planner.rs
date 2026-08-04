use crate::{Candidate, TextRange};

/// A deterministic, non-overlapping edit plan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CorrectionPlan {
    pub candidates: Vec<Candidate>,
}

impl CorrectionPlan {
    #[must_use]
    pub fn from_candidates(mut candidates: Vec<Candidate>) -> Self {
        candidates.sort_by(|left, right| {
            (left.range.start, left.range.end, left.rule_id.as_str()).cmp(&(
                right.range.start,
                right.range.end,
                right.rule_id.as_str(),
            ))
        });
        let mut accepted = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            if accepted
                .last()
                .is_some_and(|previous: &Candidate| overlaps(previous.range, candidate.range))
            {
                continue;
            }
            accepted.push(candidate);
        }
        Self {
            candidates: accepted,
        }
    }
}

fn overlaps(left: TextRange, right: TextRange) -> bool {
    left.start < right.end && right.start < left.end
}
