use crate::{DiagnosticV2, Engine, LintOutcome, SourceKind};

/// Result of the compatibility pipeline. The legacy and v2 paths share one analysis pass.
#[derive(Clone, Debug, PartialEq)]
pub struct PipelineOutcome {
    pub diagnostics: Vec<DiagnosticV2>,
    pub fixed_text: String,
    pub review_fixed_text: String,
}

/// Ordered pipeline facade around the existing deterministic engine.
///
/// New stages can be inserted behind this facade without changing the byte ranges, diagnostic
/// order, or correction previews exposed by the original [`Engine`] methods.
#[derive(Debug)]
pub struct Pipeline<'a> {
    engine: &'a Engine,
}

impl<'a> Pipeline<'a> {
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }

    #[must_use]
    pub fn check(&self, text: &str, source_kind: SourceKind) -> Vec<DiagnosticV2> {
        self.engine
            .check(text, source_kind)
            .iter()
            .map(DiagnosticV2::from_legacy)
            .collect()
    }

    #[must_use]
    pub fn check_with_fixes(
        &self,
        text: &str,
        source_kind: SourceKind,
        include_review_fixes: bool,
    ) -> PipelineOutcome {
        let legacy = self
            .engine
            .check_with_fixes(text, source_kind, include_review_fixes);
        PipelineOutcome {
            diagnostics: legacy
                .diagnostics
                .iter()
                .map(DiagnosticV2::from_legacy)
                .collect(),
            fixed_text: legacy.fixed_text,
            review_fixed_text: legacy.review_fixed_text,
        }
    }
}

impl From<LintOutcome> for PipelineOutcome {
    fn from(outcome: LintOutcome) -> Self {
        Self {
            diagnostics: outcome
                .diagnostics
                .iter()
                .map(DiagnosticV2::from_legacy)
                .collect(),
            fixed_text: outcome.fixed_text,
            review_fixed_text: outcome.review_fixed_text,
        }
    }
}
