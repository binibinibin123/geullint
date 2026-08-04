#![forbid(unsafe_code)]

//! `GeulLint`'s offline linting core.

mod analysis;
mod candidate;
mod endings;
mod lexicon;
mod matcher;
mod pipeline;
mod planner;
mod policy;
mod productive;
mod ranking;
mod source;
#[cfg(feature = "standard")]
mod standard;
mod style;
mod trace;

pub mod api;

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

use matcher::{LiteralMatcher, MatchBoundary};
pub(crate) use source::source_ranges;

pub use analysis::lattice::AnalysisLattice;
pub use analysis::phonology::{
    SyllableFeatures, compose_syllable, decompose_syllable, phonology_distance,
};
pub use analysis::{AnalyzedDocument, AnalyzedWord};
pub use api::{Candidate, DiagnosticV2, Evidence, RuleContext, Suggestion};
pub use candidate::{
    CandidateGenerator, GrammarCandidateGenerator, GrammarRule, SpacingCandidateGenerator,
    SpellingCandidateGenerator,
};
pub use lexicon::{LexiconEntry, LexiconError, StandardLexicon};
pub use pipeline::{Pipeline, PipelineOutcome};
pub use planner::CorrectionPlan;
pub use policy::FixPolicy;
pub use policy::{PolicyDecision, PolicyThresholds};
pub use ranking::{CandidateScorer, DeterministicScorer, GeulRankSmall, RankWeights};
#[cfg(feature = "standard")]
pub use standard::{StandardPipeline, StandardPipelineError, StandardPipelineOutcome};
pub use style::{StyleContext, StyleProfile};
pub use trace::{TraceEvent, TraceSink, VecTrace};

/// A half-open UTF-8 byte range in the original source text.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

/// The level assigned to a lint diagnostic.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// The rule bundle selected for a document.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// High-confidence objective rules. This is the default for CI and editors.
    #[default]
    Default,
    /// Includes the default bundle and broader objective checks.
    Strict,
    /// Includes strict rules plus subjective editorial guidance.
    Editorial,
}

impl Profile {
    const ALL: [Self; 3] = [Self::Default, Self::Strict, Self::Editorial];

    fn includes(self, required: Self) -> bool {
        matches!(
            (self, required),
            (Self::Default, Self::Default)
                | (Self::Strict, Self::Default | Self::Strict)
                | (Self::Editorial, _)
        )
    }

    fn enabled_from(required: Self) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|profile| profile.includes(required))
            .collect()
    }
}

/// Confidence assigned to a diagnostic rule after corpus validation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// Whether a suggested edit can be applied without user review.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FixSafety {
    Safe,
    Review,
    None,
}

/// Stable, user-facing metadata for a bundled rule.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMetadata {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub confidence: Confidence,
    pub default_enabled: bool,
    pub fix_safety: FixSafety,
    pub profiles: Vec<Profile>,
    pub incorrect_examples: Vec<String>,
    pub correct_examples: Vec<String>,
    pub documentation_url: String,
}

/// A deterministic, text-only lexical overlay for the bundled Korean dictionary.
///
/// The compact release asset is built from this versioned interchange format;
/// it never requires an API key or a network request while linting.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DictionaryOverlay {
    entries: BTreeMap<String, String>,
}

impl DictionaryOverlay {
    /// Parses the `geullint-overlay-v1` tab-separated interchange format.
    ///
    /// # Errors
    ///
    /// Returns an error when the version header or an entry is malformed.
    pub fn parse(source: &str) -> Result<Self, OverlayError> {
        let mut lines = source.lines();
        if lines.next() != Some("geullint-overlay-v1") {
            return Err(OverlayError::MissingVersionHeader);
        }

        let mut entries = BTreeMap::new();
        for (line_index, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            let Some((surface, part_of_speech)) = line.split_once('\t') else {
                return Err(OverlayError::InvalidEntry {
                    line: line_index + 2,
                });
            };
            if surface.is_empty() || part_of_speech.is_empty() || part_of_speech.contains('\t') {
                return Err(OverlayError::InvalidEntry {
                    line: line_index + 2,
                });
            }
            entries.insert(surface.to_owned(), part_of_speech.to_owned());
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn part_of_speech(&self, surface: &str) -> Option<&str> {
        self.entries.get(surface).map(String::as_str)
    }

    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the accepted surface forms in deterministic lexical order.
    pub fn surfaces(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

/// Reasons a dictionary overlay cannot be loaded safely.
#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("dictionary overlay must start with geullint-overlay-v1")]
    MissingVersionHeader,
    #[error("dictionary overlay entry on line {line} is malformed")]
    InvalidEntry { line: usize },
}

/// A Korean morphological token produced by the bundled offline dictionary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MorphToken {
    pub surface: String,
    pub part_of_speech: String,
    pub range: TextRange,
}

/// A deterministic Korean morphological analyzer backed by the embedded `ko-dic` data.
///
/// Construction and analysis are entirely local: the dictionary is compiled into the
/// binary and no API credential, process, or network connection is involved.
#[cfg(feature = "morphology")]
#[derive(Clone)]
pub struct MorphAnalyzer {
    tokenizer: lindera::tokenizer::Tokenizer,
    overlay: DictionaryOverlay,
}

#[cfg(feature = "morphology")]
impl MorphAnalyzer {
    /// Loads the Korean morphology dictionary embedded in the current binary.
    ///
    /// # Errors
    ///
    /// Returns an error when the embedded dictionary cannot be initialized.
    pub fn bundled() -> Result<Self, MorphError> {
        Self::with_overlay(DictionaryOverlay::default())
    }

    /// Loads the embedded dictionary and applies a deterministic project overlay.
    ///
    /// Overlay POS tags take precedence when the tokenizer emits the same surface form.
    /// No overlay source is fetched at runtime.
    ///
    /// # Errors
    ///
    /// Returns an error when the embedded dictionary cannot be initialized.
    pub fn with_overlay(overlay: DictionaryOverlay) -> Result<Self, MorphError> {
        let mut builder =
            lindera::tokenizer::TokenizerBuilder::new().map_err(MorphError::Initialization)?;
        builder.set_segmenter_dictionary("embedded://ko-dic");
        let tokenizer = builder.build().map_err(MorphError::Initialization)?;
        Ok(Self { tokenizer, overlay })
    }

    /// Splits Korean text into dictionary-backed morphemes and their POS tags.
    ///
    /// Ranges are half-open UTF-8 byte offsets in `text`, matching [`TextRange`].
    ///
    /// # Errors
    ///
    /// Returns an error when the embedded tokenizer cannot analyze the supplied text.
    pub fn analyze(&self, text: &str) -> Result<Vec<MorphToken>, MorphError> {
        let tokens = self
            .tokenizer
            .tokenize(text)
            .map_err(MorphError::Analysis)?;
        Ok(tokens
            .into_iter()
            .map(|mut token| {
                let bundled_part_of_speech =
                    token.details().first().copied().unwrap_or("UNK").to_owned();
                let surface = token.surface.into_owned();
                let part_of_speech = self
                    .overlay
                    .part_of_speech(&surface)
                    .unwrap_or(&bundled_part_of_speech)
                    .to_owned();
                MorphToken {
                    surface,
                    part_of_speech,
                    range: TextRange {
                        start: token.byte_start,
                        end: token.byte_end,
                    },
                }
            })
            .collect())
    }
}

/// Reasons the bundled Korean morphology engine cannot initialize or analyze text.
#[cfg(feature = "morphology")]
#[derive(Debug, thiserror::Error)]
pub enum MorphError {
    #[error("failed to initialize the bundled Korean morphology dictionary")]
    Initialization(#[source] lindera::error::LinderaError),
    #[error("failed to analyze Korean text with the bundled morphology dictionary")]
    Analysis(#[source] lindera::error::LinderaError),
}

/// The source type being linted.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[default]
    PlainText,
    Markdown,
    #[serde(alias = "javascript")]
    JavaScript,
    #[serde(alias = "typescript")]
    TypeScript,
    Python,
    Rust,
}

/// User-selected linting options.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LintConfig {
    pub profile: Profile,
    pub disabled_rules: Vec<String>,
    /// Project terms that are valid lexical forms in this repository.
    pub user_dictionary: Vec<String>,
    /// Terms loaded from a versioned dictionary overlay or supplied by an embedding client.
    pub dictionary_overlay: Vec<String>,
}

impl LintConfig {
    fn is_disabled(&self, rule_id: &str) -> bool {
        self.disabled_rules.iter().any(|id| id == rule_id)
    }

    fn suppresses_with_user_dictionary(&self, rule_id: &str, original: &str) -> bool {
        is_dictionary_aware(rule_id)
            && (self.user_dictionary.iter().any(|entry| entry == original)
                || self
                    .dictionary_overlay
                    .iter()
                    .any(|entry| entry == original))
    }
}

/// A lint finding, with zero or more replacement suggestions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub range: TextRange,
    pub original: String,
    pub suggestions: Vec<String>,
    pub safe_fix: bool,
}

/// Diagnostics and stable correction previews computed from one initial analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintOutcome {
    pub diagnostics: Vec<Diagnostic>,
    pub fixed_text: String,
    pub review_fixed_text: String,
}

/// A reusable linter with a fixed configuration.
#[derive(Clone, Debug)]
pub struct Engine {
    config: LintConfig,
    rule_pack_rules: Vec<LiteralRule>,
}

impl Engine {
    #[must_use]
    pub fn new(config: LintConfig) -> Self {
        Self {
            config,
            rule_pack_rules: Vec::new(),
        }
    }

    /// Returns the immutable configuration used by this engine.
    #[must_use]
    pub const fn config(&self) -> &LintConfig {
        &self.config
    }

    /// Creates an offline linter with validated versioned rule packs.
    ///
    /// # Errors
    ///
    /// Returns an error when a rule-pack ID collides with a bundled rule or another pack.
    pub fn with_rule_packs(
        config: LintConfig,
        packs: Vec<RulePack>,
    ) -> Result<Self, RulePackError> {
        let mut ids = available_rule_ids();
        let mut rule_pack_rules = Vec::new();
        for pack in packs {
            for rule in pack.rules {
                if !ids.insert(rule.id.clone()) {
                    return Err(RulePackError::DuplicateRuleId(rule.id));
                }
                rule_pack_rules.push(rule);
            }
        }
        Ok(Self {
            config,
            rule_pack_rules,
        })
    }

    #[must_use]
    pub fn check(&self, text: &str, source_kind: SourceKind) -> Vec<Diagnostic> {
        lint_text_with_rule_packs(text, source_kind, &self.config, &self.rule_pack_rules)
    }

    /// Checks a document and computes stable correction previews without repeating the initial
    /// analysis. When review fixes are not requested, both preview fields contain the safe result.
    #[must_use]
    pub fn check_with_fixes(
        &self,
        text: &str,
        source_kind: SourceKind,
        include_review_fixes: bool,
    ) -> LintOutcome {
        let diagnostics = self.check(text, source_kind);
        let first_fixed = apply_suggested_fixes(text, &diagnostics, false);
        let fixed_text = self.stabilize_after_first(text, first_fixed.clone(), source_kind, false);
        let review_fixed_text = if include_review_fixes {
            let first_review_fixed = apply_suggested_fixes(text, &diagnostics, true);
            if first_review_fixed == first_fixed {
                fixed_text.clone()
            } else {
                self.stabilize_after_first(text, first_review_fixed, source_kind, true)
            }
        } else {
            fixed_text.clone()
        };
        LintOutcome {
            diagnostics,
            fixed_text,
            review_fixed_text,
        }
    }

    /// Applies safe fixes until another pass would make no change.
    ///
    /// Chained rules can expose a second valid correction after the first edit. This method
    /// resolves those chains in one user-visible operation, while detecting custom-rule cycles
    /// and abandoning an unexpectedly long rewrite instead of returning a partially fixed text.
    #[must_use]
    pub fn fix(&self, text: &str, source_kind: SourceKind) -> String {
        self.fix_until_stable(text, source_kind, false)
    }

    /// Applies safe fixes and opt-in review suggestions until the result is stable.
    ///
    /// Review suggestions can be subjective and should only be used after an explicit user
    /// choice, such as the review checkbox in the browser demo.
    #[must_use]
    pub fn fix_with_review(&self, text: &str, source_kind: SourceKind) -> String {
        self.fix_until_stable(text, source_kind, true)
    }

    fn fix_until_stable(
        &self,
        text: &str,
        source_kind: SourceKind,
        include_review: bool,
    ) -> String {
        let diagnostics = self.check(text, source_kind);
        let first = apply_suggested_fixes(text, &diagnostics, include_review);
        self.stabilize_after_first(text, first, source_kind, include_review)
    }

    fn stabilize_after_first(
        &self,
        original: &str,
        first: String,
        source_kind: SourceKind,
        include_review: bool,
    ) -> String {
        const MAX_PASSES: usize = 32;

        if first == original {
            return first;
        }

        let original = original.to_owned();
        let mut current = first;
        let mut seen = BTreeSet::from([original.clone(), current.clone()]);

        for _ in 1..MAX_PASSES {
            let diagnostics = self.check(&current, source_kind);
            let next = apply_suggested_fixes(&current, &diagnostics, include_review);
            if next == current {
                return current;
            }
            if !seen.insert(next.clone()) {
                return original;
            }
            current = next;
        }

        original
    }

    #[must_use]
    pub fn check_document(&self, document: &DocumentSession) -> Vec<Diagnostic> {
        self.check(document.text(), document.source_kind())
    }
}

/// An in-memory document that can be incrementally updated by an editor.
#[derive(Clone, Debug)]
pub struct DocumentSession {
    text: String,
    source_kind: SourceKind,
}

impl DocumentSession {
    #[must_use]
    pub fn new(text: impl Into<String>, source_kind: SourceKind) -> Self {
        Self {
            text: text.into(),
            source_kind,
        }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn source_kind(&self) -> SourceKind {
        self.source_kind
    }

    /// Applies a UTF-8 byte edit. Invalid or non-character-boundary edits are rejected.
    ///
    /// # Errors
    ///
    /// Returns an error when the edit is out of bounds or splits a UTF-8 character.
    pub fn apply_edit(&mut self, edit: &TextEdit) -> Result<(), EditError> {
        if edit.range.start > edit.range.end || edit.range.end > self.text.len() {
            return Err(EditError::OutOfBounds);
        }
        if !self.text.is_char_boundary(edit.range.start)
            || !self.text.is_char_boundary(edit.range.end)
        {
            return Err(EditError::NotCharacterBoundary);
        }
        self.text
            .replace_range(edit.range.start..edit.range.end, &edit.replacement);
        Ok(())
    }
}

/// An edit to a [`DocumentSession`], expressed in UTF-8 bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

/// Reasons an incremental edit cannot be applied safely.
#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("the edit range is outside the document")]
    OutOfBounds,
    #[error("the edit range splits a UTF-8 character")]
    NotCharacterBoundary,
}

/// Lints text entirely in-process, without network access or telemetry.
#[must_use]
pub fn lint_text(text: &str, source_kind: SourceKind, config: &LintConfig) -> Vec<Diagnostic> {
    lint_text_with_rule_packs(text, source_kind, config, &[])
}

fn lint_text_with_rule_packs(
    text: &str,
    source_kind: SourceKind,
    config: &LintConfig,
    rule_pack_rules: &[LiteralRule],
) -> Vec<Diagnostic> {
    let document = AnalyzedDocument::new(text, source_kind);
    let source_ranges = document.source_ranges();
    let mut diagnostics = bundled_literal_diagnostics(text, source_ranges, config);
    diagnostics.extend(rule_pack_literal_diagnostics(
        text,
        source_ranges,
        config,
        rule_pack_rules,
    ));
    diagnostics.extend(native_diagnostics(text, &document, config));
    diagnostics.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then_with(|| left.range.end.cmp(&right.range.end))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });
    diagnostics
}

fn bundled_literal_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    let rules = literal_rules();
    let matcher = bundled_literal_matcher();
    let mut diagnostics = Vec::new();

    for source_range in source_ranges {
        let source = &text[source_range.start..source_range.end];
        for matched in matcher.find(source) {
            let rule = &rules[matched.rule_index];
            if rule.id == "punctuation.duplicate.comma"
                || config.is_disabled(&rule.id)
                || !config.profile.includes(rule.profile)
            {
                continue;
            }
            let replacement = &rule.replacements[matched.replacement_index];
            let original = &source[matched.start..matched.end];
            if config.suppresses_with_user_dictionary(&rule.id, original) {
                continue;
            }
            diagnostics.push(Diagnostic {
                rule_id: rule.id.clone(),
                severity: rule.severity,
                message: rule.message.clone(),
                range: TextRange {
                    start: source_range.start + matched.start,
                    end: source_range.start + matched.end,
                },
                original: original.to_owned(),
                suggestions: vec![replacement.to.clone()],
                safe_fix: rule.safe_fix,
            });
        }
    }

    diagnostics
}

fn bundled_literal_matcher() -> &'static LiteralMatcher {
    static MATCHER: OnceLock<LiteralMatcher> = OnceLock::new();
    MATCHER.get_or_init(|| LiteralMatcher::from_bundled_rules(literal_rules()))
}

fn rule_pack_literal_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
    rule_pack_rules: &[LiteralRule],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for rule in rule_pack_rules
        .iter()
        .filter(|rule| !config.is_disabled(&rule.id) && config.profile.includes(rule.profile))
    {
        for source_range in source_ranges {
            let source = &text[source_range.start..source_range.end];
            for replacement in &rule.replacements {
                for (relative_start, original) in source.match_indices(&replacement.from) {
                    let relative_end = relative_start + original.len();
                    if !replacement
                        .boundary
                        .unwrap_or(MatchBoundary::Substring)
                        .allows(source, relative_start, relative_end)
                        || config.suppresses_with_user_dictionary(&rule.id, original)
                    {
                        continue;
                    }
                    diagnostics.push(Diagnostic {
                        rule_id: rule.id.clone(),
                        severity: rule.severity,
                        message: rule.message.clone(),
                        range: TextRange {
                            start: source_range.start + relative_start,
                            end: source_range.start + relative_end,
                        },
                        original: original.to_owned(),
                        suggestions: vec![replacement.to.clone()],
                        safe_fix: rule.safe_fix,
                    });
                }
            }
        }
    }

    diagnostics
}

/// Returns the stable IDs of all bundled rules.
#[must_use]
pub fn available_rule_ids() -> BTreeSet<String> {
    let mut ids: BTreeSet<_> = literal_rules().iter().map(|rule| rule.id.clone()).collect();
    ids.extend(NATIVE_LITERALS.iter().map(|rule| rule.id.to_owned()));
    ids.extend(DYNAMIC_NATIVE_RULE_IDS.iter().map(|id| (*id).to_owned()));
    ids
}

/// Returns public metadata for a bundled rule ID.
#[must_use]
#[allow(clippy::too_many_lines)] // Keeps all literal and native metadata fallbacks in one auditable mapping.
pub fn rule_metadata(rule_id: &str) -> Option<RuleMetadata> {
    let literal_rule = literal_rules().iter().find(|rule| rule.id == rule_id);
    let native_replacements: Vec<_> = NATIVE_LITERALS
        .iter()
        .filter(|rule| rule.id == rule_id)
        .collect();
    let dynamic_metadata = DYNAMIC_NATIVE_RULE_METADATA
        .iter()
        .find(|metadata| metadata.id == rule_id);

    (literal_rule.is_some() || !native_replacements.is_empty() || dynamic_metadata.is_some()).then(
        || RuleMetadata {
            id: rule_id.to_owned(),
            title: literal_rule
                .and_then(|rule| rule.title.clone())
                .or_else(|| dynamic_metadata.map(|metadata| metadata.title.to_owned()))
                .or_else(|| curated_rule_title(rule_id).map(str::to_owned))
                .unwrap_or_else(|| humanize_rule_id(rule_id)),
            description: literal_rule
                .and_then(|rule| rule.description.clone())
                .or_else(|| dynamic_metadata.map(|metadata| metadata.description.to_owned()))
                .or_else(|| literal_rule.map(|rule| rule.message.clone()))
                .or_else(|| {
                    native_replacements
                        .first()
                        .map(|rule| rule.message.to_owned())
                })
                .unwrap_or_else(|| format!("`{rule_id}` 한국어 검사 규칙입니다.")),
            category: rule_id
                .split_once('.')
                .map_or_else(|| "other".to_owned(), |(category, _)| category.to_owned()),
            confidence: literal_rule
                .and_then(|rule| rule.confidence)
                .or_else(|| dynamic_metadata.map(|metadata| metadata.confidence))
                .unwrap_or_else(|| {
                    if rule_id == "repetition.adjacent-word" {
                        Confidence::Medium
                    } else {
                        Confidence::High
                    }
                }),
            default_enabled: literal_rule.map_or_else(
                || {
                    dynamic_metadata
                        .is_none_or(|metadata| metadata.minimum_profile == Profile::Default)
                },
                |rule| {
                    rule.default_enabled
                        .unwrap_or(rule.profile == Profile::Default)
                },
            ),
            fix_safety: literal_rule.map_or_else(
                || {
                    dynamic_metadata.map_or_else(
                        || {
                            if native_replacements.iter().all(|rule| rule.safe_fix) {
                                FixSafety::Safe
                            } else {
                                FixSafety::Review
                            }
                        },
                        |metadata| metadata.fix_safety,
                    )
                },
                |rule| {
                    if rule.safe_fix {
                        FixSafety::Safe
                    } else {
                        FixSafety::Review
                    }
                },
            ),
            profiles: literal_rule.map_or_else(
                || {
                    Profile::enabled_from(
                        dynamic_metadata
                            .map_or(Profile::Default, |metadata| metadata.minimum_profile),
                    )
                },
                |rule| Profile::enabled_from(rule.profile),
            ),
            incorrect_examples: curated_rule_examples(rule_id)
                .map(|(incorrect, _)| vec![incorrect.to_owned()])
                .or_else(|| dynamic_metadata.map(|metadata| vec![metadata.incorrect.to_owned()]))
                .or_else(|| {
                    literal_rule.map(|rule| {
                        rule.examples.as_ref().map_or_else(
                            || {
                                rule.replacements
                                    .iter()
                                    .map(|replacement| replacement.from.clone())
                                    .collect()
                            },
                            |examples| examples.incorrect.clone(),
                        )
                    })
                })
                .unwrap_or_else(|| {
                    native_replacements
                        .iter()
                        .map(|rule| rule.from.to_string())
                        .collect()
                }),
            correct_examples: curated_rule_examples(rule_id)
                .map(|(_, correct)| vec![correct.to_owned()])
                .or_else(|| dynamic_metadata.map(|metadata| vec![metadata.correct.to_owned()]))
                .or_else(|| {
                    literal_rule.map(|rule| {
                        rule.examples.as_ref().map_or_else(
                            || {
                                rule.replacements
                                    .iter()
                                    .map(|replacement| replacement.to.clone())
                                    .collect()
                            },
                            |examples| examples.correct.clone(),
                        )
                    })
                })
                .unwrap_or_else(|| {
                    native_replacements
                        .iter()
                        .map(|rule| rule.to.to_string())
                        .collect()
                }),
            documentation_url: format!(
                "https://github.com/binibinibin123/geullint/blob/master/docs/rules.md#{rule_id}"
            ),
        },
    )
}

/// Returns every bundled rule's public metadata in stable ID order.
#[must_use]
pub fn rule_catalog() -> Vec<RuleMetadata> {
    available_rule_ids()
        .into_iter()
        .filter_map(|rule_id| rule_metadata(&rule_id))
        .collect()
}

fn curated_rule_title(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        "grammar.negation.an-before-predicate" => Some("부정 부사 ‘안’"),
        "grammar.negation.ji-anh" => Some("‘-지 않다’ 띄어쓰기"),
        "grammar.particle.duplicate" => Some("조사 중복"),
        "punctuation.no-space-before-mark" => Some("문장 부호 앞 띄어쓰기"),
        "repetition.ending" => Some("종결 표현 반복"),
        "spacing.dependent-noun.geot" => Some("의존 명사 ‘것’ 띄어쓰기"),
        "spacing.dependent-noun.jeok" => Some("의존 명사 ‘적’ 띄어쓰기"),
        "spacing.dependent-noun.jul" => Some("의존 명사 ‘줄’ 띄어쓰기"),
        "spacing.dependent-noun.jung" => Some("의존 명사 ‘중’ 띄어쓰기"),
        "spacing.dependent-noun.ppun" => Some("의존 명사 ‘뿐’ 띄어쓰기"),
        "spacing.dependent-noun.su" => Some("의존 명사 ‘수’ 띄어쓰기"),
        "spacing.dependent-noun.ttae" => Some("의존 명사 ‘때’ 띄어쓰기"),
        "spacing.fixed.ppunman-anira" => Some("‘뿐만 아니라’ 띄어쓰기"),
        "spacing.fixed.su-bakke" => Some("‘수밖에’ 붙여쓰기"),
        "spelling.adverb.i-hi" => Some("부사 ‘-이/-히’ 표기"),
        "spelling.confusable.oraen-oraet" => Some("‘오랜/오랫-’ 표기"),
        "spelling.confusable.wen-waen" => Some("‘웬/왠’ 구별"),
        "spelling.conjugation.boe-bwae" => Some("‘봬요’ 표기"),
        "spelling.conjugation.dwaet" => Some("‘됐’ 표기"),
        "spelling.lexical.anseong-matchum" => Some("안성맞춤 표기"),
        "spelling.lexical.chojeom" => Some("초점 표기"),
        "spelling.lexical.daega" => Some("대가 표기"),
        "spelling.lexical.dodaeche" => Some("도대체 표기"),
        "spelling.lexical.eoieopda" => Some("어이없다 표기"),
        "spelling.lexical.eojjaetdeun" => Some("어쨌든 표기"),
        "spelling.lexical.gaesu" => Some("개수 표기"),
        "spelling.lexical.geokkuro" => Some("거꾸로 표기"),
        "spelling.lexical.geumse" => Some("금세 표기"),
        "spelling.lexical.gop-ppaegi" => Some("곱빼기 표기"),
        "spelling.lexical.hamatteomyeon" => Some("하마터면 표기"),
        "spelling.lexical.huihanhada" => Some("희한하다 표기"),
        "spelling.lexical.jjagipgi" => Some("짜깁기 표기"),
        "spelling.lexical.jjigae" => Some("찌개 표기"),
        "spelling.lexical.tongjjaero" => Some("통째로 표기"),
        "spelling.lexical.yeokhal" => Some("역할 표기"),
        "spelling.lexical.yukgaejang" => Some("육개장 표기"),
        "spelling.loanword.curated" => Some("표준 외래어 표기"),
        "style.redundancy.gajang-choego" => Some("‘가장 최고’ 의미 중복"),
        _ => None,
    }
}

fn curated_rule_examples(rule_id: &str) -> Option<(&'static str, &'static str)> {
    match rule_id {
        "grammar.particle.duplicate" => Some(("자료를를 확인했다", "자료를 확인했다")),
        _ => None,
    }
}

fn humanize_rule_id(rule_id: &str) -> String {
    rule_id
        .rsplit('.')
        .next()
        .unwrap_or(rule_id)
        .split('-')
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(characters).collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Applies only safe, non-overlapping diagnostics to the original UTF-8 text.
#[must_use]
pub fn apply_safe_fixes(text: &str, diagnostics: &[Diagnostic]) -> String {
    apply_suggested_fixes(text, diagnostics, false)
}

fn apply_suggested_fixes(text: &str, diagnostics: &[Diagnostic], include_review: bool) -> String {
    let mut candidates: Vec<_> = diagnostics
        .iter()
        .filter_map(|diagnostic| {
            (diagnostic.safe_fix || include_review)
                .then(|| {
                    diagnostic
                        .suggestions
                        .first()
                        .map(|replacement| (diagnostic, replacement))
                })
                .flatten()
        })
        .filter(|(diagnostic, _)| {
            diagnostic.range.start <= diagnostic.range.end
                && diagnostic.range.end <= text.len()
                && text.is_char_boundary(diagnostic.range.start)
                && text.is_char_boundary(diagnostic.range.end)
        })
        .collect();
    candidates.sort_by(|(left, _), (right, _)| {
        left.range
            .start
            .cmp(&right.range.start)
            .then_with(|| left.range.end.cmp(&right.range.end))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });

    let mut accepted = Vec::new();
    let mut previous_end = 0;
    for candidate @ (diagnostic, _) in candidates {
        if diagnostic.range.start >= previous_end {
            previous_end = diagnostic.range.end;
            accepted.push(candidate);
        }
    }

    let mut fixed = text.to_owned();
    for (diagnostic, replacement) in accepted.into_iter().rev() {
        fixed.replace_range(diagnostic.range.start..diagnostic.range.end, replacement);
    }
    fixed
}

fn is_dictionary_aware(rule_id: &str) -> bool {
    rule_id.starts_with("spelling.lexical.") || rule_id == "spelling.loanword.curated"
}

struct NativeLiteral {
    id: &'static str,
    severity: Severity,
    message: &'static str,
    from: &'static str,
    to: &'static str,
    safe_fix: bool,
}

struct AllomorphRule {
    id: &'static str,
    message: &'static str,
    incorrect_particle: &'static str,
    correct_particle: &'static str,
    should_replace: fn(char) -> bool,
}

struct NativeRuleMetadata {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    incorrect: &'static str,
    correct: &'static str,
    confidence: Confidence,
    minimum_profile: Profile,
    fix_safety: FixSafety,
}

const NATIVE_LITERALS: &[NativeLiteral] = &[
    NativeLiteral {
        id: "grammar.negation.ji-anh",
        severity: Severity::Warning,
        message: "‘-지 않았다’처럼 보조 용언은 붙여 씁니다.",
        from: "지 안았다",
        to: "지 않았다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.negation.an-before-predicate",
        severity: Severity::Warning,
        message: "부정 부사 ‘안’을 사용하세요.",
        from: "않 간다",
        to: "안 간다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.particle.duplicate",
        severity: Severity::Warning,
        message: "조사가 중복된 것 같습니다.",
        from: "를를",
        to: "를",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.particle.duplicate",
        severity: Severity::Warning,
        message: "조사가 중복된 것 같습니다.",
        from: "을을",
        to: "을",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.particle.duplicate",
        severity: Severity::Warning,
        message: "조사가 중복된 것 같습니다.",
        from: "은은",
        to: "은",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.particle.duplicate",
        severity: Severity::Warning,
        message: "조사가 중복된 것 같습니다.",
        from: "는는",
        to: "는",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.su",
        severity: Severity::Warning,
        message: "의존 명사 ‘수’는 앞말과 띄어 씁니다.",
        from: "할수 있다",
        to: "할 수 있다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.su",
        severity: Severity::Warning,
        message: "의존 명사 ‘수’는 앞말과 띄어 씁니다.",
        from: "끝낼수",
        to: "끝낼 수",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.su",
        severity: Severity::Warning,
        message: "의존 명사 ‘수’는 앞말과 띄어 씁니다.",
        from: "알수없다",
        to: "알 수 없다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.su",
        severity: Severity::Warning,
        message: "의존 명사 ‘수’는 앞말과 띄어 씁니다.",
        from: "알수 있다",
        to: "알 수 있다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.geot",
        severity: Severity::Warning,
        message: "의존 명사 ‘것’은 앞말과 띄어 씁니다.",
        from: "좋을것 같다",
        to: "좋을 것 같다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.geot",
        severity: Severity::Warning,
        message: "의존 명사 ‘것’은 앞말과 띄어 씁니다.",
        from: "올것 같다",
        to: "올 것 같다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.geot",
        severity: Severity::Warning,
        message: "의존 명사 ‘것’은 앞말과 띄어 씁니다.",
        from: "될것이다",
        to: "될 것이다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.geot",
        severity: Severity::Warning,
        message: "의존 명사 ‘거’는 앞말과 띄어 씁니다.",
        from: "할거야",
        to: "할 거야",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.jeok",
        severity: Severity::Warning,
        message: "의존 명사 ‘적’은 앞말과 띄어 씁니다.",
        from: "본적 있다",
        to: "본 적 있다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.jeok",
        severity: Severity::Warning,
        message: "의존 명사 ‘적’은 앞말과 띄어 씁니다.",
        from: "만난적 있다",
        to: "만난 적 있다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.jeok",
        severity: Severity::Warning,
        message: "의존 명사 ‘적’은 앞말과 띄어 씁니다.",
        from: "해본적 없다",
        to: "해본 적 없다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.jul",
        severity: Severity::Warning,
        message: "의존 명사 ‘줄’은 앞말과 띄어 씁니다.",
        from: "알줄 안다",
        to: "알 줄 안다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.ppun",
        severity: Severity::Warning,
        message: "의존 명사 ‘뿐’은 앞말과 띄어 씁니다.",
        from: "기다릴뿐이다",
        to: "기다릴 뿐이다",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.ttae",
        severity: Severity::Warning,
        message: "의존 명사 ‘때’는 앞말과 띄어 씁니다.",
        from: "만날때",
        to: "만날 때",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.dependent-noun.jung",
        severity: Severity::Warning,
        message: "의존 명사 ‘중’은 앞말과 띄어 씁니다.",
        from: "하는중",
        to: "하는 중",
        safe_fix: true,
    },
    NativeLiteral {
        id: "spacing.fixed.su-bakke",
        severity: Severity::Warning,
        message: "‘수밖에’는 붙여 씁니다.",
        from: "할 수 밖에",
        to: "할 수밖에",
        safe_fix: true,
    },
    NativeLiteral {
        id: "punctuation.no-space-before-mark",
        severity: Severity::Warning,
        message: "문장 부호 앞에는 띄어쓰지 않습니다.",
        from: " .",
        to: ".",
        safe_fix: true,
    },
    NativeLiteral {
        id: "punctuation.no-space-before-mark",
        severity: Severity::Warning,
        message: "문장 부호 앞에는 띄어쓰지 않습니다.",
        from: " !",
        to: "!",
        safe_fix: true,
    },
    NativeLiteral {
        id: "punctuation.no-space-before-mark",
        severity: Severity::Warning,
        message: "문장 부호 앞에는 띄어쓰지 않습니다.",
        from: " ?",
        to: "?",
        safe_fix: true,
    },
    NativeLiteral {
        id: "repetition.ending",
        severity: Severity::Info,
        message: "어미가 반복된 것 같습니다.",
        from: "했습니다습니다",
        to: "했습니다",
        safe_fix: true,
    },
];

const DYNAMIC_NATIVE_RULE_IDS: &[&str] = &[
    "grammar.copula.anieyo",
    "grammar.conjugation.doe-to-dwae",
    "grammar.conjugation.dwae-to-doe",
    "grammar.ending.colloquial-yong",
    "grammar.ending.deun-choice",
    "grammar.ending.euryeo",
    "grammar.ending.euryeo-context",
    "grammar.ending.seumnida",
    "grammar.ending.sipsio",
    "grammar.negation.anh-doe",
    "grammar.particle.topic-allomorph",
    "grammar.particle.subject-allomorph",
    "grammar.particle.object-allomorph",
    "grammar.particle.comitative-allomorph",
    "grammar.particle.instrumental-allomorph",
    "punctuation.space-after-comma",
    "punctuation.space-after-sentence-mark",
    "repetition.adjacent-word",
    "spacing.dependent-noun.beop",
    "spacing.dependent-noun.chae",
    "spacing.dependent-noun.daero",
    "spacing.dependent-noun.de",
    "spacing.dependent-noun.deut",
    "spacing.dependent-noun.mankeum",
    "spacing.dependent-noun.ri",
];

const DYNAMIC_NATIVE_RULE_METADATA: &[NativeRuleMetadata] = &[
    NativeRuleMetadata {
        id: "grammar.copula.anieyo",
        title: "‘아니에요’ 표기",
        description: "‘아니다’에 ‘-에요’가 붙은 활용형을 ‘아니에요’로 바로잡습니다.",
        incorrect: "아니예요",
        correct: "아니에요",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.conjugation.doe-to-dwae",
        title: "‘되’와 ‘돼’ 활용 구별",
        description: "‘되서’, ‘되요’와 잘못 줄인 ‘됀’, ‘됄’, ‘됌’을 올바른 활용으로 고칩니다.",
        incorrect: "됀",
        correct: "된",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.conjugation.dwae-to-doe",
        title: "‘돼’와 ‘되’ 활용 구별",
        description: "‘돼게’, ‘돼면서’, ‘돼도록’처럼 어미 앞에서는 ‘되’를 씁니다.",
        incorrect: "돼게",
        correct: "되게",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.ending.colloquial-yong",
        title: "표준 종결어미 ‘-요’",
        description: "편집 문체에서 ‘해용’, ‘세용’을 표준 종결어미로 검토하도록 안내합니다.",
        incorrect: "감사해용",
        correct: "감사해요",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Editorial,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.ending.deun-choice",
        title: "선택의 ‘-든지’",
        description: "선택을 나타내는 연결 어미 ‘-든지’를 ‘-던지’와 구별합니다.",
        incorrect: "커피던지 차던지",
        correct: "커피든지 차든지",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.ending.euryeo",
        title: "의도·조건의 ‘-려고/-려면’",
        description: "검증된 활용형에서 불필요하게 덧붙은 ‘ㄹ’을 바로잡습니다.",
        incorrect: "먹을려고",
        correct: "먹으려고",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.ending.euryeo-context",
        title: "‘갈려고/갈려면’ 문맥 검토",
        description: "‘갈려고’, ‘갈려면’을 문맥에 따라 ‘가려고’, ‘가려면’으로 검토합니다.",
        incorrect: "갈려고",
        correct: "가려고",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Strict,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.ending.seumnida",
        title: "현대 표준 ‘-습니다/-ㅂ니다’",
        description: "옛 표기 ‘-읍니다/-읍니까’를 받침에 맞는 현대 표준 활용으로 바로잡습니다.",
        incorrect: "읽읍니다",
        correct: "읽습니다",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.ending.sipsio",
        title: "높임 명령형 ‘-십시오’",
        description: "높임 명령형의 잘못된 ‘-십시요’를 ‘-십시오’로 바로잡습니다.",
        incorrect: "확인하십시요",
        correct: "확인하십시오",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "grammar.negation.anh-doe",
        title: "부정 부사 ‘안’과 ‘되다’",
        description: "‘않되다/않돼다’처럼 잘못 쓴 부정을 ‘안 되다’ 계열로 바로잡습니다.",
        incorrect: "않됩니다",
        correct: "안 됩니다",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.de",
        title: "의존 명사 ‘데’ 띄어쓰기",
        description: "관형형 뒤에서 장소·경우를 나타내는 ‘데’의 띄어쓰기를 검토합니다.",
        incorrect: "묵을데가",
        correct: "묵을 데가",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.chae",
        title: "의존 명사 ‘채’ 띄어쓰기",
        description: "관형형 뒤에서 상태를 나타내는 ‘채’의 띄어쓰기를 검토합니다.",
        incorrect: "입은채로",
        correct: "입은 채로",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.deut",
        title: "의존 명사 ‘듯’ 띄어쓰기",
        description: "관형형 뒤에서 짐작을 나타내는 ‘듯’의 띄어쓰기를 검토합니다.",
        incorrect: "모르는듯하다",
        correct: "모르는 듯하다",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.mankeum",
        title: "의존 명사 ‘만큼’ 띄어쓰기",
        description: "관형형 뒤에서 정도를 나타내는 ‘만큼’의 띄어쓰기를 검토합니다.",
        incorrect: "먹을만큼",
        correct: "먹을 만큼",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.daero",
        title: "의존 명사 ‘대로’ 띄어쓰기",
        description: "관형형 뒤에서 양상·방식을 나타내는 ‘대로’의 띄어쓰기를 검토합니다.",
        incorrect: "들은대로",
        correct: "들은 대로",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.beop",
        title: "의존 명사 ‘법’ 띄어쓰기",
        description: "관형형 뒤에서 일반적인 이치를 나타내는 ‘법’의 띄어쓰기를 검토합니다.",
        incorrect: "사는법이다",
        correct: "사는 법이다",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "spacing.dependent-noun.ri",
        title: "의존 명사 ‘리’ 띄어쓰기",
        description: "관형형 뒤에서 가능성을 나타내는 ‘리’의 띄어쓰기를 검토합니다.",
        incorrect: "잊을리가 없다",
        correct: "잊을 리가 없다",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.particle.topic-allomorph",
        title: "보조사 ‘은/는’",
        description: "앞말의 받침에 맞춰 보조사 ‘은/는’을 선택합니다.",
        incorrect: "책는",
        correct: "책은",
        confidence: Confidence::High,
        minimum_profile: Profile::Strict,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.particle.subject-allomorph",
        title: "주격 조사 ‘이/가’",
        description: "앞말의 받침에 맞춰 주격 조사 ‘이/가’를 선택합니다.",
        incorrect: "나무이",
        correct: "나무가",
        confidence: Confidence::High,
        minimum_profile: Profile::Strict,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.particle.object-allomorph",
        title: "목적격 조사 ‘을/를’",
        description: "앞말의 받침에 맞춰 목적격 조사 ‘을/를’을 선택합니다.",
        incorrect: "책를",
        correct: "책을",
        confidence: Confidence::High,
        minimum_profile: Profile::Strict,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.particle.comitative-allomorph",
        title: "접속 조사 ‘과/와’",
        description: "앞말의 받침에 맞춰 접속 조사 ‘과/와’를 선택합니다.",
        incorrect: "책와",
        correct: "책과",
        confidence: Confidence::High,
        minimum_profile: Profile::Strict,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "grammar.particle.instrumental-allomorph",
        title: "부사격 조사 ‘으로/로’",
        description: "앞말의 받침에 맞춰 부사격 조사 ‘으로/로’를 선택합니다.",
        incorrect: "책로",
        correct: "책으로",
        confidence: Confidence::High,
        minimum_profile: Profile::Strict,
        fix_safety: FixSafety::Review,
    },
    NativeRuleMetadata {
        id: "punctuation.space-after-comma",
        title: "쉼표 뒤 띄어쓰기",
        description: "쉼표 뒤에 이어지는 한국어 문장을 한 칸 띄웁니다.",
        incorrect: "사과,배",
        correct: "사과, 배",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "punctuation.space-after-sentence-mark",
        title: "문장 부호 뒤 띄어쓰기",
        description: "마침표·느낌표·물음표 뒤의 다음 문장을 한 칸 띄웁니다.",
        incorrect: "끝났다.다음",
        correct: "끝났다. 다음",
        confidence: Confidence::High,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Safe,
    },
    NativeRuleMetadata {
        id: "repetition.adjacent-word",
        title: "인접 단어 반복",
        description: "같은 단어가 바로 이어서 반복된 부분을 찾습니다.",
        incorrect: "문서를 문서를",
        correct: "문서를",
        confidence: Confidence::Medium,
        minimum_profile: Profile::Default,
        fix_safety: FixSafety::Review,
    },
];

const TOPIC_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.topic-allomorph",
    message: "은/는 조사를 앞말의 받침에 맞추세요.",
    incorrect_particle: "는",
    correct_particle: "은",
    should_replace: has_final_consonant,
};

const OBJECT_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.object-allomorph",
    message: "을/를 조사를 앞말의 받침에 맞추세요.",
    incorrect_particle: "를",
    correct_particle: "을",
    should_replace: has_final_consonant,
};

const COMITATIVE_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.comitative-allomorph",
    message: "과/와 조사를 앞말의 받침에 맞추세요.",
    incorrect_particle: "와",
    correct_particle: "과",
    should_replace: has_final_consonant,
};

const INSTRUMENTAL_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.instrumental-allomorph",
    message: "‘로’는 받침이 있는 말 뒤에서 ‘으로’가 됩니다. (ㄹ 받침 제외)",
    incorrect_particle: "로",
    correct_particle: "으로",
    should_replace: has_non_rieul_final_consonant,
};

const TOPIC_REVERSE_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.topic-allomorph",
    message: "은/는 조사를 앞말의 받침에 맞추세요.",
    incorrect_particle: "은",
    correct_particle: "는",
    should_replace: |character| !has_final_consonant(character),
};

const OBJECT_REVERSE_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.object-allomorph",
    message: "을/를 조사를 앞말의 받침에 맞추세요.",
    incorrect_particle: "을",
    correct_particle: "를",
    should_replace: |character| !has_final_consonant(character),
};

const COMITATIVE_REVERSE_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.comitative-allomorph",
    message: "과/와 조사를 앞말의 받침에 맞추세요.",
    incorrect_particle: "과",
    correct_particle: "와",
    should_replace: |character| !has_final_consonant(character),
};

const INSTRUMENTAL_REVERSE_ALLOMORPH_RULE: AllomorphRule = AllomorphRule {
    id: "grammar.particle.instrumental-allomorph",
    message: "‘으로’는 모음 뒤에서 ‘로’가 됩니다.",
    incorrect_particle: "으로",
    correct_particle: "로",
    should_replace: |character| !has_final_consonant(character),
};

fn native_diagnostics(
    text: &str,
    document: &AnalyzedDocument,
    config: &LintConfig,
) -> Vec<Diagnostic> {
    let source_ranges = document.source_ranges();
    let mut diagnostics = native_literal_diagnostics(text, document, config);
    diagnostics.extend(ending_diagnostics(document, config));
    diagnostics.extend(productive_diagnostics(text, document, config));
    diagnostics.extend(parallel_deun_diagnostics(text, source_ranges, config));
    diagnostics.extend(particle_diagnostics(text, source_ranges, config));
    diagnostics.extend(punctuation_diagnostics(text, source_ranges, config));
    diagnostics.extend(repeated_word_diagnostics(text, source_ranges, config));
    diagnostics
}

fn productive_diagnostics(
    text: &str,
    document: &AnalyzedDocument,
    config: &LintConfig,
) -> Vec<Diagnostic> {
    let source_ranges = document.source_ranges();
    let mut source_range_index = 0;
    let mut diagnostics = Vec::new();

    for word in document.words() {
        while source_ranges
            .get(source_range_index)
            .is_some_and(|range| range.end < word.range.end)
        {
            source_range_index += 1;
        }
        let following_text = source_ranges
            .get(source_range_index)
            .filter(|range| range.start <= word.range.start && word.range.end <= range.end)
            .map_or("", |range| &text[word.range.end..range.end]);

        productive::for_each_match(&word.surface, following_text, |matched| {
            if ending_rule_enabled(config, matched.rule_id) {
                diagnostics.push(native_diagnostic(
                    matched.rule_id,
                    Severity::Warning,
                    matched.message,
                    word.range,
                    &word.surface,
                    &matched.replacement,
                    dynamic_rule_is_safe(matched.rule_id),
                ));
            }
        });
    }

    diagnostics
}

fn ending_diagnostics(document: &AnalyzedDocument, config: &LintConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if ending_rule_enabled(config, "grammar.conjugation.doe-to-dwae") {
        diagnostics.extend(document.words().iter().flat_map(|word| {
            endings::doe_to_dwae_edits(&word.surface)
                .into_iter()
                .map(|edit| {
                    native_diagnostic(
                        "grammar.conjugation.doe-to-dwae",
                        Severity::Warning,
                        "‘되/돼’의 활용과 준말 표기를 바로잡으세요.",
                        TextRange {
                            start: word.range.start + edit.start,
                            end: word.range.start + edit.end,
                        },
                        &word.surface[edit.start..edit.end],
                        &edit.replacement,
                        dynamic_rule_is_safe("grammar.conjugation.doe-to-dwae"),
                    )
                })
        }));
    }

    if ending_rule_enabled(config, "grammar.conjugation.dwae-to-doe") {
        diagnostics.extend(document.words().iter().filter_map(|word| {
            endings::correct_dwae_to_doe(&word.surface).map(|suggestion| {
                native_diagnostic(
                    "grammar.conjugation.dwae-to-doe",
                    Severity::Warning,
                    "‘돼’ 뒤에 이 어미가 오면 ‘되’로 씁니다.",
                    word.range,
                    &word.surface,
                    &suggestion,
                    dynamic_rule_is_safe("grammar.conjugation.dwae-to-doe"),
                )
            })
        }));
    }

    if ending_rule_enabled(config, "grammar.ending.euryeo") {
        diagnostics.extend(document.words().iter().filter_map(|word| {
            endings::correct_known_euryeo(&word.surface).map(|suggestion| {
                native_diagnostic(
                    "grammar.ending.euryeo",
                    Severity::Warning,
                    "이 활용형에서는 ‘-려고/-려면’을 사용하세요.",
                    word.range,
                    &word.surface,
                    &suggestion,
                    dynamic_rule_is_safe("grammar.ending.euryeo"),
                )
            })
        }));
    }

    if ending_rule_enabled(config, "grammar.ending.euryeo-context") {
        diagnostics.extend(document.words().iter().filter_map(|word| {
            endings::review_context_euryeo(&word.surface).map(|suggestion| {
                native_diagnostic(
                    "grammar.ending.euryeo-context",
                    Severity::Warning,
                    "문맥에 따라 ‘가려고/가려면’으로 고치는 것을 검토하세요.",
                    word.range,
                    &word.surface,
                    &suggestion,
                    dynamic_rule_is_safe("grammar.ending.euryeo-context"),
                )
            })
        }));
    }

    if ending_rule_enabled(config, "grammar.ending.colloquial-yong") {
        diagnostics.extend(document.words().iter().filter_map(|word| {
            endings::review_colloquial_yong(&word.surface).map(|suggestion| {
                native_diagnostic(
                    "grammar.ending.colloquial-yong",
                    Severity::Info,
                    "편집 문체에서는 표준 종결어미 ‘-요’를 검토하세요.",
                    word.range,
                    &word.surface,
                    &suggestion,
                    dynamic_rule_is_safe("grammar.ending.colloquial-yong"),
                )
            })
        }));
    }

    diagnostics
}

fn ending_rule_enabled(config: &LintConfig, rule_id: &str) -> bool {
    !config.is_disabled(rule_id)
        && DYNAMIC_NATIVE_RULE_METADATA
            .iter()
            .find(|metadata| metadata.id == rule_id)
            .is_some_and(|metadata| config.profile.includes(metadata.minimum_profile))
}

fn dynamic_rule_is_safe(rule_id: &str) -> bool {
    DYNAMIC_NATIVE_RULE_METADATA
        .iter()
        .find(|metadata| metadata.id == rule_id)
        .is_some_and(|metadata| metadata.fix_safety == FixSafety::Safe)
}

fn native_literal_diagnostics(
    text: &str,
    document: &AnalyzedDocument,
    config: &LintConfig,
) -> Vec<Diagnostic> {
    NATIVE_LITERALS
        .iter()
        .filter(|rule| {
            rule.id != "punctuation.no-space-before-mark" && !config.is_disabled(rule.id)
        })
        .flat_map(|rule| {
            document
                .source_ranges()
                .iter()
                .flat_map(move |source_range| {
                    text[source_range.start..source_range.end]
                        .match_indices(rule.from)
                        .filter_map(move |(relative_start, matched)| {
                            let matched_range = TextRange {
                                start: source_range.start + relative_start,
                                end: source_range.start + relative_start + matched.len(),
                            };
                            let range = matched_range;
                            let original = matched;
                            let replacement = rule.to;

                            if rule.id == "grammar.particle.duplicate"
                                && !document.words().iter().any(|word| {
                                    word.range.start < range.start && word.range.end == range.end
                                })
                            {
                                return None;
                            }

                            Some(native_diagnostic(
                                rule.id,
                                rule.severity,
                                rule.message,
                                range,
                                original,
                                replacement,
                                rule.safe_fix,
                            ))
                        })
                })
        })
        .collect()
}

fn particle_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    if !config.profile.includes(Profile::Strict) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    diagnostics.extend(allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &TOPIC_ALLOMORPH_RULE,
    ));
    diagnostics.extend(allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &OBJECT_ALLOMORPH_RULE,
    ));
    diagnostics.extend(allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &COMITATIVE_ALLOMORPH_RULE,
    ));
    diagnostics.extend(allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &INSTRUMENTAL_ALLOMORPH_RULE,
    ));
    diagnostics.extend(reverse_allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &TOPIC_REVERSE_ALLOMORPH_RULE,
    ));
    diagnostics.extend(reverse_allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &OBJECT_REVERSE_ALLOMORPH_RULE,
    ));
    diagnostics.extend(reverse_allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &COMITATIVE_REVERSE_ALLOMORPH_RULE,
    ));
    diagnostics.extend(reverse_allomorph_diagnostics(
        text,
        source_ranges,
        config,
        &INSTRUMENTAL_REVERSE_ALLOMORPH_RULE,
    ));
    if !config.is_disabled("grammar.particle.subject-allomorph") {
        for source_range in source_ranges {
            for (relative_start, _) in
                text[source_range.start..source_range.end].match_indices('이')
            {
                let start = source_range.start + relative_start;
                let end = start + '이'.len_utf8();
                let Some((_, previous)) = text[..start].char_indices().last() else {
                    continue;
                };
                let previous_start = preceding_hangul_word_start(text, start);
                if is_hangul_syllable(previous)
                    && !has_final_consonant(previous)
                    && is_likely_subject_noun(preceding_hangul_word(text, start))
                    && text[end..].chars().next().is_none_or(is_particle_boundary)
                {
                    diagnostics.push(native_diagnostic(
                        "grammar.particle.subject-allomorph",
                        Severity::Warning,
                        "이/가 조사를 앞말의 받침에 맞추세요.",
                        TextRange {
                            start: previous_start,
                            end,
                        },
                        &text[previous_start..end],
                        &format!("{}가", preceding_hangul_word(text, start)),
                        false,
                    ));
                }
            }
        }
    }
    diagnostics
}

fn allomorph_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
    rule: &AllomorphRule,
) -> Vec<Diagnostic> {
    if config.is_disabled(rule.id) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for source_range in source_ranges {
        for (relative_start, _) in
            text[source_range.start..source_range.end].match_indices(rule.incorrect_particle)
        {
            let start = source_range.start + relative_start;
            let end = start + rule.incorrect_particle.len();
            let Some((_, previous)) = text[..start].char_indices().last() else {
                continue;
            };
            let previous_start = preceding_hangul_word_start(text, start);
            if is_hangul_syllable(previous)
                && (rule.should_replace)(previous)
                && previous.to_string() != rule.incorrect_particle
                && (rule.id != "grammar.particle.topic-allomorph"
                    || is_likely_topic_noun(preceding_hangul_word(text, start)))
            {
                diagnostics.push(native_diagnostic(
                    rule.id,
                    Severity::Warning,
                    rule.message,
                    TextRange {
                        start: previous_start,
                        end,
                    },
                    &text[previous_start..end],
                    &format!(
                        "{}{}",
                        preceding_hangul_word(text, start),
                        rule.correct_particle
                    ),
                    false,
                ));
            }
        }
    }
    diagnostics
}

fn reverse_allomorph_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
    rule: &AllomorphRule,
) -> Vec<Diagnostic> {
    if config.is_disabled(rule.id) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for source_range in source_ranges {
        for (relative_start, _) in
            text[source_range.start..source_range.end].match_indices(rule.incorrect_particle)
        {
            let start = source_range.start + relative_start;
            let end = start + rule.incorrect_particle.len();
            let Some(previous) = text[..start].chars().next_back() else {
                continue;
            };
            let noun = preceding_hangul_word(text, start);
            if !is_likely_particle_noun(noun)
                || !(rule.should_replace)(previous)
                || !text[end..].chars().next().is_none_or(is_particle_boundary)
            {
                continue;
            }

            let word_start = preceding_hangul_word_start(text, start);
            diagnostics.push(native_diagnostic(
                rule.id,
                Severity::Warning,
                rule.message,
                TextRange {
                    start: word_start,
                    end,
                },
                &text[word_start..end],
                &format!("{}{}", noun, rule.correct_particle),
                false,
            ));
        }
    }
    diagnostics
}

fn parallel_deun_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    const RULE_ID: &str = "grammar.ending.deun-choice";
    if config.is_disabled(RULE_ID) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for source_range in source_ranges {
        let source = &text[source_range.start..source_range.end];
        let mut candidates = Vec::new();
        for (relative_start, _) in source.match_indices("던지") {
            let suffix_start = source_range.start + relative_start;
            let word_start = preceding_hangul_word_start(text, suffix_start);
            candidates.push((word_start, suffix_start + "던지".len()));
        }
        for pair in candidates.windows(2) {
            let (_, first_end) = pair[0];
            let (second_start, _) = pair[1];
            if text[first_end..second_start]
                .chars()
                .all(char::is_whitespace)
            {
                for (start, end) in pair {
                    let original = &text[*start..*end];
                    let suggestion = format!("{}든지", &text[*start..*end - "던지".len()]);
                    diagnostics.push(native_diagnostic(
                        RULE_ID,
                        Severity::Warning,
                        "선택을 나열할 때는 ‘-든지’를 사용하세요.",
                        TextRange {
                            start: *start,
                            end: *end,
                        },
                        original,
                        &suggestion,
                        true,
                    ));
                }
            }
        }
    }
    diagnostics
}

fn repeated_word_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    const RULE_ID: &str = "repetition.adjacent-word";
    if config.is_disabled(RULE_ID) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for source_range in source_ranges {
        let mut words = hangul_words(text, *source_range).into_iter().peekable();
        while let Some((start, end)) = words.next() {
            let Some((next_start, next_end)) = words.peek().copied() else {
                break;
            };
            if text[start..end] == text[next_start..next_end]
                && text[end..next_start].chars().all(char::is_whitespace)
                && !is_intentional_adjacent_repeat(&text[start..end])
            {
                diagnostics.push(native_diagnostic(
                    RULE_ID,
                    Severity::Info,
                    "같은 낱말이 연속으로 반복된 것 같습니다.",
                    TextRange {
                        start,
                        end: next_end,
                    },
                    &text[start..next_end],
                    &text[start..end],
                    false,
                ));
                words.next();
            }
        }
    }
    diagnostics
}

fn is_intentional_adjacent_repeat(word: &str) -> bool {
    // Korean distributive adverbial: “그때 그때” is intentionally repeated.
    matches!(word, "그때")
}

fn hangul_words(text: &str, range: TextRange) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut word_start = None;
    for (relative_index, character) in text[range.start..range.end].char_indices() {
        let index = range.start + relative_index;
        if is_hangul_syllable(character) {
            word_start.get_or_insert(index);
        } else if let Some(start) = word_start.take() {
            words.push((start, index));
        }
    }
    if let Some(start) = word_start {
        words.push((start, range.end));
    }
    words
}

fn preceding_hangul_word(text: &str, before: usize) -> &str {
    let start = preceding_hangul_word_start(text, before);
    &text[start..before]
}

fn preceding_hangul_word_start(text: &str, before: usize) -> usize {
    text[..before]
        .char_indices()
        .rev()
        .find_map(|(index, character)| {
            (!is_hangul_syllable(character)).then_some(index + character.len_utf8())
        })
        .unwrap_or(0)
}

fn is_likely_topic_noun(word: &str) -> bool {
    is_likely_particle_noun(word)
}

fn is_likely_subject_noun(word: &str) -> bool {
    matches!(
        word,
        "나무" | "사과" | "바다" | "학교" | "의자" | "모자" | "친구" | "연필" | "동생"
    )
}

fn is_likely_particle_noun(word: &str) -> bool {
    matches!(
        word,
        "책" | "연필"
            | "사과"
            | "친구"
            | "동생"
            | "의자"
            | "학교"
            | "나무"
            | "문서"
            | "파일"
            | "값"
            | "글"
            | "댓글"
            | "사용자"
            | "프로젝트"
            | "코드"
            | "자료"
            | "결과"
            | "정보"
            | "문장"
            | "문제"
            | "시간"
            | "회의"
            | "방법"
            | "이유"
            | "작업"
            | "계획"
            | "서비스"
            | "제품"
            | "기능"
            | "화면"
            | "규칙"
            | "단어"
            | "문법"
            | "검사기"
    )
}

fn punctuation_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for source_range in source_ranges {
        let source = &text[source_range.start..source_range.end];

        if !config.is_disabled("punctuation.no-space-before-mark") {
            append_space_before_mark_diagnostics(text, *source_range, &mut diagnostics);
        }

        for (relative_start, character) in source.char_indices() {
            let start = source_range.start + relative_start;
            let end = start + character.len_utf8();
            let relative_end = relative_start + character.len_utf8();
            let next = source[relative_end..].chars().next();

            if character == ',' {
                if source[..relative_start].ends_with(',') {
                    continue;
                }

                let mut comma_run_end = relative_end;
                while source.as_bytes().get(comma_run_end) == Some(&b',') {
                    comma_run_end += 1;
                }
                let after_run = source[comma_run_end..].chars().next();
                let duplicate_commas = comma_run_end - relative_start > 1;

                if duplicate_commas && !config.is_disabled("punctuation.duplicate.comma") {
                    let range = TextRange {
                        start,
                        end: source_range.start + comma_run_end,
                    };
                    let replacement = if !config.is_disabled("punctuation.space-after-comma")
                        && after_run.is_some_and(is_hangul_syllable)
                    {
                        ", "
                    } else {
                        ","
                    };
                    diagnostics.push(native_diagnostic(
                        "punctuation.duplicate.comma",
                        Severity::Warning,
                        "쉼표가 중복되었습니다.",
                        range,
                        &text[range.start..range.end],
                        replacement,
                        true,
                    ));
                    continue;
                }

                if !config.is_disabled("punctuation.space-after-comma")
                    && after_run.is_some_and(is_hangul_syllable)
                {
                    let comma_start = source_range.start + comma_run_end - 1;
                    diagnostics.push(native_diagnostic(
                        "punctuation.space-after-comma",
                        Severity::Warning,
                        "쉼표 뒤에는 띄어쓰기를 넣으세요.",
                        TextRange {
                            start: comma_start,
                            end: comma_start + 1,
                        },
                        ",",
                        ", ",
                        true,
                    ));
                }
            }

            if !config.is_disabled("punctuation.space-after-sentence-mark")
                && matches!(character, '.' | '!' | '?')
                && next.is_some_and(is_hangul_syllable)
                && source[..relative_start]
                    .chars()
                    .next_back()
                    .is_none_or(|previous| !previous.is_ascii_digit())
            {
                diagnostics.push(native_diagnostic(
                    "punctuation.space-after-sentence-mark",
                    Severity::Warning,
                    "문장 부호 뒤에는 띄어쓰기를 넣으세요.",
                    TextRange { start, end },
                    &text[start..end],
                    &format!("{character} "),
                    true,
                ));
            }
        }
    }
    diagnostics
}

fn append_space_before_mark_diagnostics(
    text: &str,
    source_range: TextRange,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let source = &text[source_range.start..source_range.end];
    let mut relative_start = 0;
    while relative_start < source.len() {
        let character = source[relative_start..]
            .chars()
            .next()
            .expect("offset stays inside source text");
        if !matches!(character, ' ' | '\t') {
            relative_start += character.len_utf8();
            continue;
        }

        let mut relative_end = relative_start + character.len_utf8();
        while relative_end < source.len() {
            let next = source[relative_end..]
                .chars()
                .next()
                .expect("offset stays inside source text");
            if !matches!(next, ' ' | '\t') {
                break;
            }
            relative_end += next.len_utf8();
        }

        if source[relative_end..]
            .chars()
            .next()
            .is_some_and(|next| matches!(next, '.' | '!' | '?'))
        {
            let range = TextRange {
                start: source_range.start + relative_start,
                end: source_range.start + relative_end,
            };
            diagnostics.push(native_diagnostic(
                "punctuation.no-space-before-mark",
                Severity::Warning,
                "문장 부호 앞에는 띄어쓰지 않습니다.",
                range,
                &text[range.start..range.end],
                "",
                true,
            ));
        }
        relative_start = relative_end;
    }
}

fn native_diagnostic(
    rule_id: &str,
    severity: Severity,
    message: &str,
    range: TextRange,
    original: &str,
    suggestion: &str,
    safe_fix: bool,
) -> Diagnostic {
    Diagnostic {
        rule_id: rule_id.to_owned(),
        severity,
        message: message.to_owned(),
        range,
        original: original.to_owned(),
        suggestions: vec![suggestion.to_owned()],
        safe_fix,
    }
}

fn is_hangul_syllable(character: char) -> bool {
    ('가'..='힣').contains(&character)
}

fn is_particle_boundary(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            ',' | '.' | '!' | '?' | '…' | ':' | ';' | ')' | ']' | '}' | '”' | '’'
        )
}

fn has_final_consonant(character: char) -> bool {
    is_hangul_syllable(character) && !(character as u32 - '가' as u32).is_multiple_of(28)
}

fn has_rieul_final(character: char) -> bool {
    is_hangul_syllable(character) && ((character as u32 - '가' as u32) % 28 == 8)
}

fn has_non_rieul_final_consonant(character: char) -> bool {
    has_final_consonant(character) && !has_rieul_final(character)
}

/// A validated, versioned offline YAML rule pack.
#[derive(Clone, Debug)]
pub struct RulePack {
    rules: Vec<LiteralRule>,
}

impl RulePack {
    /// Parses a `version: 1`, `language: ko` YAML rule pack without any network access.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed YAML, unsupported metadata, duplicate IDs, or empty rules.
    pub fn parse(source: &str) -> Result<Self, RulePackError> {
        let file: RuleFile = serde_yaml::from_str(source).map_err(RulePackError::InvalidYaml)?;
        validate_rule_file(&file)?;
        Ok(Self { rules: file.rules })
    }
}

/// Reasons a local rule pack cannot be used safely.
#[derive(Debug, thiserror::Error)]
pub enum RulePackError {
    #[error("rule pack YAML is invalid")]
    InvalidYaml(#[source] serde_yaml::Error),
    #[error("rule pack version {0} is unsupported; expected version 1")]
    UnsupportedVersion(u8),
    #[error("rule pack language {0:?} is unsupported; expected ko")]
    UnsupportedLanguage(String),
    #[error("rule pack must contain at least one rule")]
    EmptyRuleSet,
    #[error("rule ID {0:?} is duplicated or collides with another loaded rule")]
    DuplicateRuleId(String),
    #[error("rule {id:?} is invalid: {reason}")]
    InvalidRule { id: String, reason: &'static str },
}

#[derive(Debug, Deserialize)]
struct RuleFile {
    version: u8,
    language: String,
    rules: Vec<LiteralRule>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiteralRule {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    severity: Severity,
    message: String,
    #[serde(default)]
    confidence: Option<Confidence>,
    #[serde(default)]
    profile: Profile,
    #[serde(default)]
    default_enabled: Option<bool>,
    safe_fix: bool,
    replacements: Vec<Replacement>,
    #[serde(default)]
    examples: Option<RuleExamples>,
}

#[derive(Clone, Debug, Deserialize)]
struct RuleExamples {
    incorrect: Vec<String>,
    correct: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Replacement {
    from: String,
    to: String,
    #[serde(default)]
    boundary: Option<MatchBoundary>,
}

fn literal_rules() -> &'static [LiteralRule] {
    const BUNDLED_RULE_SOURCES: &[&str] = &[
        include_str!("../../../rules/ko-basic.yaml"),
        include_str!("../../../rules/catalog/spacing.yaml"),
        include_str!("../../../rules/catalog/confusable.yaml"),
        include_str!("../../../rules/catalog/grammar.yaml"),
        include_str!("../../../rules/catalog/punctuation.yaml"),
        include_str!("../../../rules/catalog/style.yaml"),
        include_str!("../../../rules/catalog/technical.yaml"),
        include_str!("../../../rules/catalog/advanced.yaml"),
        include_str!("../../../rules/catalog/curated-core.yaml"),
    ];

    static RULES: OnceLock<Vec<LiteralRule>> = OnceLock::new();
    RULES
        .get_or_init(|| {
            let mut rules = Vec::new();
            for source in BUNDLED_RULE_SOURCES {
                let file: RuleFile =
                    serde_yaml::from_str(source).expect("bundled Korean rule YAML must be valid");
                validate_rule_file(&file).expect("bundled Korean rule YAML must be valid");
                rules.extend(file.rules);
            }

            let merged = RuleFile {
                version: 1,
                language: "ko".to_owned(),
                rules,
            };
            validate_rule_file(&merged)
                .expect("merged bundled Korean rule catalogue must be valid");
            merged.rules
        })
        .as_slice()
}

fn validate_rule_file(file: &RuleFile) -> Result<(), RulePackError> {
    if file.version != 1 {
        return Err(RulePackError::UnsupportedVersion(file.version));
    }
    if file.language != "ko" {
        return Err(RulePackError::UnsupportedLanguage(file.language.clone()));
    }
    if file.rules.is_empty() {
        return Err(RulePackError::EmptyRuleSet);
    }

    let mut ids = BTreeSet::new();
    let mut catalogue_sources = BTreeSet::new();
    for rule in &file.rules {
        if rule.id.trim().is_empty() {
            return Err(RulePackError::InvalidRule {
                id: rule.id.clone(),
                reason: "rule ID must not be empty",
            });
        }
        if !ids.insert(rule.id.clone()) {
            return Err(RulePackError::DuplicateRuleId(rule.id.clone()));
        }
        if rule.message.trim().is_empty() {
            return Err(RulePackError::InvalidRule {
                id: rule.id.clone(),
                reason: "message must not be empty",
            });
        }
        if rule.replacements.is_empty() {
            return Err(RulePackError::InvalidRule {
                id: rule.id.clone(),
                reason: "at least one replacement is required",
            });
        }

        let mut sources = BTreeSet::new();
        for replacement in &rule.replacements {
            if replacement.from.is_empty() || replacement.to.is_empty() {
                return Err(RulePackError::InvalidRule {
                    id: rule.id.clone(),
                    reason: "replacement forms must not be empty",
                });
            }
            if !sources.insert(replacement.from.as_str()) {
                return Err(RulePackError::InvalidRule {
                    id: rule.id.clone(),
                    reason: "replacement source forms must be unique per rule",
                });
            }
            if !catalogue_sources.insert(replacement.from.as_str()) {
                return Err(RulePackError::InvalidRule {
                    id: rule.id.clone(),
                    reason: "replacement source forms must be unique across the catalogue",
                });
            }
        }
    }
    Ok(())
}
