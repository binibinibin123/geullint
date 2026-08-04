use crate::{
    AnalyzedDocument, Confidence, Diagnostic, FixSafety, LintConfig, Severity, SourceKind,
    TextRange,
};
use serde::{Deserialize, Serialize};

/// A compact, machine-readable reason attached to a candidate or suggestion.
/// Evidence intentionally stores features rather than source text so tracing does not leak a
/// document into logs or telemetry.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub code: String,
    pub value: String,
    pub score: f64,
}

impl Evidence {
    #[must_use]
    pub fn new(code: impl Into<String>, value: impl Into<String>, score: f64) -> Self {
        Self {
            code: code.into(),
            value: value.into(),
            score: score.clamp(0.0, 1.0),
        }
    }
}

/// One ranked replacement for a diagnostic.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub text: String,
    pub safety: FixSafety,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

/// Versioned diagnostic shape used by new integrations while the original [`Diagnostic`] API
/// remains available for compatibility.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticV2 {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub range: TextRange,
    pub original: String,
    pub suggestions: Vec<Suggestion>,
    pub safety: FixSafety,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

impl DiagnosticV2 {
    #[must_use]
    pub fn from_legacy(diagnostic: &Diagnostic) -> Self {
        let safety = if diagnostic.suggestions.is_empty() {
            FixSafety::None
        } else if diagnostic.safe_fix {
            FixSafety::Safe
        } else {
            FixSafety::Review
        };
        let confidence = if diagnostic.safe_fix {
            Confidence::High
        } else if diagnostic.suggestions.is_empty() {
            Confidence::Low
        } else {
            Confidence::Medium
        };
        let evidence = vec![Evidence::new(
            "legacy-rule",
            diagnostic.rule_id.clone(),
            if diagnostic.safe_fix { 1.0 } else { 0.5 },
        )];
        let suggestions = diagnostic
            .suggestions
            .iter()
            .map(|text| Suggestion {
                text: text.clone(),
                safety,
                confidence,
                evidence: evidence.clone(),
            })
            .collect();
        Self {
            rule_id: diagnostic.rule_id.clone(),
            severity: diagnostic.severity,
            message: diagnostic.message.clone(),
            range: diagnostic.range,
            original: diagnostic.original.clone(),
            suggestions,
            safety,
            confidence,
            evidence,
        }
    }
}

/// Immutable context passed to analysis and candidate stages.
#[derive(Debug)]
pub struct RuleContext<'a> {
    text: &'a str,
    source_kind: SourceKind,
    document: &'a AnalyzedDocument,
    config: &'a LintConfig,
}

impl<'a> RuleContext<'a> {
    #[must_use]
    pub fn new(
        text: &'a str,
        source_kind: SourceKind,
        document: &'a AnalyzedDocument,
        config: &'a LintConfig,
    ) -> Self {
        Self {
            text,
            source_kind,
            document,
            config,
        }
    }

    #[must_use]
    pub fn text(&self) -> &'a str {
        self.text
    }

    #[must_use]
    pub fn source_kind(&self) -> SourceKind {
        self.source_kind
    }

    #[must_use]
    pub fn document(&self) -> &'a AnalyzedDocument {
        self.document
    }

    #[must_use]
    pub fn config(&self) -> &'a LintConfig {
        self.config
    }
}

/// A bounded correction candidate before final policy arbitration.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub rule_id: String,
    pub range: TextRange,
    pub original: String,
    pub replacement: String,
    pub score: f32,
    pub evidence: Vec<Evidence>,
}

impl Candidate {
    #[must_use]
    pub fn new(
        rule_id: impl Into<String>,
        range: TextRange,
        original: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            range,
            original: original.into(),
            replacement: replacement.into(),
            score: 0.0,
            evidence: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_evidence(mut self, evidence: Evidence) -> Self {
        self.evidence.push(evidence);
        self
    }
}
