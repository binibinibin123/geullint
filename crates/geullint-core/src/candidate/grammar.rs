use super::CandidateGenerator;
use crate::{Candidate, Evidence, RuleContext};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrammarRule {
    pub id: String,
    pub incorrect: String,
    pub replacement: String,
}

#[derive(Clone, Debug, Default)]
pub struct GrammarCandidateGenerator {
    rules: Vec<GrammarRule>,
    max_candidates: usize,
}

impl GrammarCandidateGenerator {
    #[must_use]
    pub fn new(rules: Vec<GrammarRule>, max_candidates: usize) -> Self {
        Self {
            rules,
            max_candidates: max_candidates.max(1),
        }
    }
}

impl CandidateGenerator for GrammarCandidateGenerator {
    fn generate(&self, context: &RuleContext<'_>) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        for word in context.document().words() {
            for rule in &self.rules {
                if word.surface != rule.incorrect {
                    continue;
                }
                candidates.push(
                    Candidate::new(&rule.id, word.range, &word.surface, &rule.replacement)
                        .with_evidence(Evidence::new("grammar-rule", &rule.id, 1.0))
                        .with_score(1.0),
                );
            }
        }
        candidates.truncate(self.max_candidates);
        candidates
    }
}
