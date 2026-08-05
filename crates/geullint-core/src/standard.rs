use crate::{
    AnalyzedDocument, Candidate, CandidateGenerator, ContextRanker, DiagnosticV2, Engine,
    FixSafety, GeulRankSmall, LintConfig, RuleContext, SourceKind, SpacingCandidateGenerator,
    SpellingCandidateGenerator, StandardLexicon, Suggestion,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_STANDARD_SUGGESTIONS: usize = 8;

/// Result of the opt-in standard pipeline.
///
/// Candidates from the standard lexicon are deliberately exposed as Review suggestions until
/// an independent release holdout calibrates their precision. Legacy compact fixes keep their
/// existing safe/review behavior and are the only edits included in the preview texts.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardPipelineOutcome {
    pub diagnostics: Vec<DiagnosticV2>,
    pub fixed_text: String,
    pub review_fixed_text: String,
}

/// Standard candidate pipeline shared by native integrations.
///
/// This is intentionally opt-in behind the `standard` feature. It wires the versioned lexicon,
/// bounded spelling/spacing generators, and deterministic ranker together without changing the
/// byte-for-byte compact engine contract.
#[derive(Debug)]
pub struct StandardPipeline {
    engine: Engine,
    spelling: SpellingCandidateGenerator,
    spacing: SpacingCandidateGenerator,
    ranker: GeulRankSmall,
    context_ranker: Option<ContextRanker>,
}

impl StandardPipeline {
    #[must_use]
    pub fn new(engine: Engine, lexicon: StandardLexicon, ranker: GeulRankSmall) -> Self {
        Self {
            engine,
            spelling: SpellingCandidateGenerator::new(lexicon.clone(), 32),
            spacing: SpacingCandidateGenerator::new(lexicon, 32),
            ranker,
            context_ranker: None,
        }
    }

    /// Loads the checked-in standard lexicon and portable ranker artifact.
    ///
    /// # Errors
    ///
    /// Returns an error if either versioned bundled asset is malformed.
    pub fn bundled(config: LintConfig) -> Result<Self, StandardPipelineError> {
        let lexicon = StandardLexicon::bundled().map_err(StandardPipelineError::Lexicon)?;
        let ranker = GeulRankSmall::bundled().map_err(StandardPipelineError::Ranker)?;
        Ok(Self::new(Engine::new(config), lexicon, ranker))
    }

    /// Load the experimental learned context ranker without making it the default path.
    ///
    /// Its candidates remain Review-only; the independent holdout gate must pass before a
    /// caller should promote any confidence to Safe.
    ///
    /// # Errors
    ///
    /// Returns an error when a bundled lexicon, deterministic ranker, or context model is
    /// malformed.
    pub fn bundled_with_context(config: LintConfig) -> Result<Self, StandardPipelineError> {
        let mut pipeline = Self::bundled(config)?;
        pipeline.context_ranker =
            Some(ContextRanker::bundled().map_err(StandardPipelineError::ContextRanker)?);
        Ok(pipeline)
    }

    /// Attach the experimental learned context ranker to an existing pipeline.
    ///
    /// This keeps caller-provided rule packs and configuration intact while preserving the
    /// explicit opt-in boundary around the learned model.
    #[must_use]
    pub fn with_context_ranker(mut self, context_ranker: ContextRanker) -> Self {
        self.context_ranker = Some(context_ranker);
        self
    }

    #[must_use]
    pub fn check(&self, text: &str, source_kind: SourceKind) -> Vec<DiagnosticV2> {
        let mut diagnostics = self
            .engine
            .check(text, source_kind)
            .iter()
            .map(DiagnosticV2::from_legacy)
            .collect::<Vec<_>>();

        let document = AnalyzedDocument::new(text, source_kind);
        let context = RuleContext::new(text, source_kind, &document, self.engine.config());
        let mut candidates = self.spelling.generate(&context);
        candidates.extend(self.spacing.generate(&context));
        if let Some(context_ranker) = &self.context_ranker {
            for candidate in &mut candidates {
                candidate.score = context_ranker.score(text, &candidate_text(text, candidate));
            }
            sort_candidates(&mut candidates);
        } else {
            self.ranker.rank(&mut candidates, &context);
        }

        let mut occupied = diagnostics
            .iter()
            .filter_map(|diagnostic| {
                diagnostic.suggestions.first().map(|suggestion| {
                    (
                        diagnostic.range.start,
                        diagnostic.range.end,
                        suggestion.text.clone(),
                    )
                })
            })
            .collect::<BTreeSet<_>>();

        let mut grouped = BTreeMap::<(usize, usize, String, String), Vec<Candidate>>::new();
        for candidate in candidates {
            if self.engine.config().is_disabled(&candidate.rule_id)
                || candidate.original == candidate.replacement
                || !occupied.insert((
                    candidate.range.start,
                    candidate.range.end,
                    candidate.replacement.clone(),
                ))
            {
                continue;
            }
            grouped
                .entry((
                    candidate.range.start,
                    candidate.range.end,
                    candidate.rule_id.clone(),
                    candidate.original.clone(),
                ))
                .or_default()
                .push(candidate);
        }

        for ((start, end, rule_id, original), candidates) in grouped {
            let Some(top) = candidates.first() else {
                continue;
            };
            let confidence = self.confidence(top.score);
            let evidence = top.evidence.clone();
            let mut seen_replacements = BTreeSet::new();
            let suggestions = candidates
                .into_iter()
                .filter(|candidate| seen_replacements.insert(candidate.replacement.clone()))
                .take(MAX_STANDARD_SUGGESTIONS)
                .map(|candidate| Suggestion {
                    text: candidate.replacement,
                    safety: FixSafety::Review,
                    confidence: self.confidence(candidate.score),
                    evidence: candidate.evidence,
                })
                .collect();
            diagnostics.push(DiagnosticV2 {
                rule_id,
                severity: crate::Severity::Info,
                message: "표준 사전 후보입니다. 문맥을 확인한 뒤 적용하세요.".to_owned(),
                range: crate::TextRange { start, end },
                original,
                suggestions,
                safety: FixSafety::Review,
                confidence,
                evidence,
            });
        }

        diagnostics.sort_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then_with(|| left.range.end.cmp(&right.range.end))
                .then_with(|| left.rule_id.cmp(&right.rule_id))
        });
        diagnostics
    }

    #[must_use]
    pub fn check_with_fixes(
        &self,
        text: &str,
        source_kind: SourceKind,
        include_review_fixes: bool,
    ) -> StandardPipelineOutcome {
        let legacy = self
            .engine
            .check_with_fixes(text, source_kind, include_review_fixes);
        let diagnostics = self.check(text, source_kind);
        let review_fixed_text = if include_review_fixes {
            apply_review_suggestions(text, &diagnostics)
        } else {
            legacy.fixed_text.clone()
        };
        StandardPipelineOutcome {
            diagnostics,
            fixed_text: legacy.fixed_text,
            review_fixed_text,
        }
    }

    #[must_use]
    pub const fn ranker(&self) -> GeulRankSmall {
        self.ranker
    }

    #[must_use]
    pub const fn has_context_ranker(&self) -> bool {
        self.context_ranker.is_some()
    }

    fn confidence(&self, score: f32) -> crate::Confidence {
        self.context_ranker.as_ref().map_or_else(
            || self.ranker.confidence(score),
            |ranker| ranker.confidence(score),
        )
    }
}

fn candidate_text(text: &str, candidate: &Candidate) -> String {
    let mut result = text.to_owned();
    result.replace_range(
        candidate.range.start..candidate.range.end,
        &candidate.replacement,
    );
    result
}

fn sort_candidates(candidates: &mut [Candidate]) {
    candidates.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.range.start.cmp(&right.range.start))
            .then_with(|| left.range.end.cmp(&right.range.end))
            .then_with(|| left.replacement.cmp(&right.replacement))
    });
}

/// Applies the first Review-or-Safe suggestion from one analysis pass.
///
/// Standard candidates are deliberately never included in `fixed_text`; this helper is only
/// used for the explicit review preview so the browser and native callers can show one complete
/// corrected sentence without re-running analysis or silently promoting a candidate to Safe.
fn apply_review_suggestions(text: &str, diagnostics: &[DiagnosticV2]) -> String {
    let mut candidates = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.safety != FixSafety::None)
        .filter_map(|diagnostic| {
            if !is_safe_review_preview(diagnostic) {
                return None;
            }
            let suggestion = diagnostic.suggestions.first()?;
            let range = diagnostic.range;
            (range.start <= range.end
                && range.end <= text.len()
                && text.is_char_boundary(range.start)
                && text.is_char_boundary(range.end))
            .then_some((range.start, range.end, suggestion.text.clone()))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut accepted = Vec::new();
    let mut previous_end = 0;
    for candidate in candidates {
        if candidate.0 >= previous_end {
            previous_end = candidate.1;
            accepted.push(candidate);
        }
    }

    let mut fixed = text.to_owned();
    for (start, end, replacement) in accepted.into_iter().rev() {
        fixed.replace_range(start..end, &replacement);
    }
    fixed
}

fn is_safe_review_preview(diagnostic: &DiagnosticV2) -> bool {
    diagnostic.safety == FixSafety::Safe
}

#[derive(Debug, thiserror::Error)]
pub enum StandardPipelineError {
    #[error("standard lexicon asset is invalid: {0}")]
    Lexicon(crate::LexiconError),
    #[error("standard ranker asset is invalid: {0}")]
    Ranker(String),
    #[error("context ranker asset is invalid: {0}")]
    ContextRanker(String),
}
