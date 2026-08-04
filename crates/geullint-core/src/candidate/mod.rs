mod grammar;
mod spacing;
mod spelling;

use crate::{Candidate, RuleContext};

/// Generates bounded correction candidates from an immutable analysis context.
pub trait CandidateGenerator {
    fn generate(&self, context: &RuleContext<'_>) -> Vec<Candidate>;
}

pub use grammar::{GrammarCandidateGenerator, GrammarRule};
pub use spacing::SpacingCandidateGenerator;
pub use spelling::SpellingCandidateGenerator;
