use crate::{Confidence, FixSafety, StyleProfile};
use serde::{Deserialize, Serialize};

/// User-visible action selected after safety and confidence calibration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FixPolicy {
    Safe,
    Review,
    Abstain,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyDecision {
    pub policy: FixPolicy,
    pub confidence: Confidence,
    pub score: f32,
    pub reason: String,
}

impl PolicyDecision {
    #[must_use]
    pub fn new(policy: FixPolicy, confidence: Confidence, reason: impl Into<String>) -> Self {
        Self {
            policy,
            confidence,
            score: 0.0,
            reason: reason.into(),
        }
    }
}

/// Thresholds calibrated against independent validation data. The default intentionally leaves
/// a dead band between review and safe automation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolicyThresholds {
    pub safe_score: f32,
    pub review_score: f32,
}

impl Default for PolicyThresholds {
    fn default() -> Self {
        Self {
            safe_score: 0.99,
            review_score: 0.60,
        }
    }
}

impl PolicyThresholds {
    #[must_use]
    pub fn decide(
        self,
        family: &str,
        safe_fix: bool,
        confidence: Confidence,
        score: f32,
        style: StyleProfile,
    ) -> PolicyDecision {
        let score = score.clamp(0.0, 1.0);
        if !safe_fix {
            return decision(
                FixPolicy::Abstain,
                confidence,
                score,
                "candidate is not safe to apply",
            );
        }
        let risky_family = family.starts_with("proper")
            || family.starts_with("name")
            || family.starts_with("style")
            || family.starts_with("register");
        if risky_family
            || matches!(
                style,
                StyleProfile::Formal | StyleProfile::Technical | StyleProfile::Code
            )
        {
            return decision(
                FixPolicy::Review,
                confidence,
                score,
                "context or family requires review",
            );
        }
        if confidence == Confidence::High && score >= self.safe_score {
            return decision(
                FixPolicy::Safe,
                confidence,
                score,
                "calibrated score clears the safe threshold",
            );
        }
        if score >= self.review_score {
            return decision(
                FixPolicy::Review,
                confidence,
                score,
                "calibrated score is in the review band",
            );
        }
        decision(
            FixPolicy::Review,
            confidence,
            score,
            "score is below the safe threshold",
        )
    }
}

fn decision(policy: FixPolicy, confidence: Confidence, score: f32, reason: &str) -> PolicyDecision {
    PolicyDecision {
        policy,
        confidence,
        score,
        reason: reason.to_owned(),
    }
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
