use crate::{Confidence, FixSafety};

/// User-visible action selected after safety and confidence calibration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixPolicy {
    Safe,
    Review,
    Abstain,
}

impl FixPolicy {
    #[must_use]
    pub fn from_safety(safe_fix: bool, confidence: Confidence) -> Self {
        if !safe_fix {
            return Self::Abstain;
        }
        match confidence {
            Confidence::High => Self::Safe,
            Confidence::Medium | Confidence::Low => Self::Review,
        }
    }

    #[must_use]
    pub fn from_fix_safety(safety: FixSafety, confidence: Confidence) -> Self {
        match safety {
            FixSafety::Safe => Self::from_safety(true, confidence),
            FixSafety::Review => Self::Review,
            FixSafety::None => Self::Abstain,
        }
    }
}
