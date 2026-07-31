#![forbid(unsafe_code)]

//! `GeulLint`'s offline linting core.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

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
#[cfg(feature = "morphology")]
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
    let source_ranges = source_ranges(text, source_kind);
    let mut diagnostics: Vec<_> = literal_rules()
        .iter()
        .chain(rule_pack_rules)
        .filter(|rule| !config.is_disabled(&rule.id) && config.profile.includes(rule.profile))
        .flat_map(|rule| {
            source_ranges.iter().flat_map(move |source_range| {
                rule.replacements.iter().flat_map(move |replacement| {
                    text[source_range.start..source_range.end]
                        .match_indices(&replacement.from)
                        .filter(move |(_, original)| {
                            !config.suppresses_with_user_dictionary(&rule.id, original)
                        })
                        .map(move |(relative_start, original)| Diagnostic {
                            rule_id: rule.id.clone(),
                            severity: rule.severity,
                            message: rule.message.clone(),
                            range: TextRange {
                                start: source_range.start + relative_start,
                                end: source_range.start + relative_start + original.len(),
                            },
                            original: original.to_owned(),
                            suggestions: vec![replacement.to.clone()],
                            safe_fix: rule.safe_fix,
                        })
                })
            })
        })
        .collect();
    diagnostics.extend(native_diagnostics(text, &source_ranges, config));
    diagnostics.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then_with(|| left.range.end.cmp(&right.range.end))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });
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
                .unwrap_or_else(|| humanize_rule_id(rule_id)),
            description: literal_rule
                .and_then(|rule| rule.description.clone())
                .or_else(|| dynamic_metadata.map(|metadata| metadata.description.to_owned()))
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
                || dynamic_metadata.is_none_or(|metadata| metadata.default_enabled),
                |rule| {
                    rule.default_enabled
                        .unwrap_or(rule.profile == Profile::Default)
                },
            ),
            fix_safety: if rule_is_safe(rule_id) {
                FixSafety::Safe
            } else {
                FixSafety::Review
            },
            profiles: literal_rules()
                .iter()
                .find(|rule| rule.id == rule_id)
                .map_or_else(
                    || Profile::enabled_from(Profile::Default),
                    |rule| Profile::enabled_from(rule.profile),
                ),
            incorrect_examples: literal_rule.map_or_else(
                || {
                    dynamic_metadata.map_or_else(
                        || {
                            native_replacements
                                .iter()
                                .map(|rule| rule.from.to_string())
                                .collect()
                        },
                        |metadata| vec![metadata.incorrect.to_owned()],
                    )
                },
                |rule| {
                    rule.examples.as_ref().map_or_else(
                        || {
                            rule.replacements
                                .iter()
                                .map(|replacement| replacement.from.clone())
                                .collect()
                        },
                        |examples| examples.incorrect.clone(),
                    )
                },
            ),
            correct_examples: literal_rule.map_or_else(
                || {
                    dynamic_metadata.map_or_else(
                        || {
                            native_replacements
                                .iter()
                                .map(|rule| rule.to.to_string())
                                .collect()
                        },
                        |metadata| vec![metadata.correct.to_owned()],
                    )
                },
                |rule| {
                    rule.examples.as_ref().map_or_else(
                        || {
                            rule.replacements
                                .iter()
                                .map(|replacement| replacement.to.clone())
                                .collect()
                        },
                        |examples| examples.correct.clone(),
                    )
                },
            ),
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
    let mut candidates: Vec<_> = diagnostics
        .iter()
        .filter_map(|diagnostic| {
            diagnostic
                .safe_fix
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

fn rule_is_safe(rule_id: &str) -> bool {
    literal_rules()
        .iter()
        .any(|rule| rule.id == rule_id && rule.safe_fix)
        || NATIVE_LITERALS
            .iter()
            .any(|rule| rule.id == rule_id && rule.safe_fix)
        || (DYNAMIC_NATIVE_RULE_IDS.contains(&rule_id) && rule_id != "repetition.adjacent-word")
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
    default_enabled: bool,
}

const NATIVE_LITERALS: &[NativeLiteral] = &[
    NativeLiteral {
        id: "grammar.conjugation.doe-to-dwae",
        severity: Severity::Warning,
        message: "‘되서’는 ‘돼서’로 쓰는 것이 맞습니다.",
        from: "되서",
        to: "돼서",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.doe-to-dwae",
        severity: Severity::Warning,
        message: "‘되도’는 ‘돼도’로 쓰는 것이 맞습니다.",
        from: "되도",
        to: "돼도",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.doe-to-dwae",
        severity: Severity::Warning,
        message: "‘되요’는 ‘돼요’로 쓰는 것이 맞습니다.",
        from: "되요",
        to: "돼요",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.doe-to-dwae",
        severity: Severity::Warning,
        message: "‘되야’는 ‘돼야’로 쓰는 것이 맞습니다.",
        from: "되야",
        to: "돼야",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.dwae-to-doe",
        severity: Severity::Warning,
        message: "‘돼면’은 ‘되면’으로 쓰는 것이 맞습니다.",
        from: "돼면",
        to: "되면",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.dwae-to-doe",
        severity: Severity::Warning,
        message: "‘돼고’는 ‘되고’로 쓰는 것이 맞습니다.",
        from: "돼고",
        to: "되고",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.dwae-to-doe",
        severity: Severity::Warning,
        message: "‘돼는’은 ‘되는’으로 쓰는 것이 맞습니다.",
        from: "돼는",
        to: "되는",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.conjugation.dwae-to-doe",
        severity: Severity::Warning,
        message: "‘됄’은 ‘될’로 쓰는 것이 맞습니다.",
        from: "됄",
        to: "될",
        safe_fix: true,
    },
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
        id: "grammar.ending.euryeo",
        severity: Severity::Warning,
        message: "‘-려고’를 사용하세요.",
        from: "할려고",
        to: "하려고",
        safe_fix: true,
    },
    NativeLiteral {
        id: "grammar.ending.euryeo",
        severity: Severity::Warning,
        message: "‘-려면’을 사용하세요.",
        from: "할려면",
        to: "하려면",
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
    "grammar.ending.deun-choice",
    "grammar.particle.topic-allomorph",
    "grammar.particle.subject-allomorph",
    "grammar.particle.object-allomorph",
    "grammar.particle.comitative-allomorph",
    "grammar.particle.instrumental-allomorph",
    "punctuation.space-after-comma",
    "punctuation.space-after-sentence-mark",
    "repetition.adjacent-word",
];

const DYNAMIC_NATIVE_RULE_METADATA: &[NativeRuleMetadata] = &[
    NativeRuleMetadata {
        id: "grammar.ending.deun-choice",
        title: "선택의 ‘-든지’",
        description: "선택을 나타내는 연결 어미 ‘-든지’를 ‘-던지’와 구별합니다.",
        incorrect: "커피던지 차던지",
        correct: "커피든지 차든지",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "grammar.particle.topic-allomorph",
        title: "보조사 ‘은/는’",
        description: "앞말의 받침에 맞춰 보조사 ‘은/는’을 선택합니다.",
        incorrect: "책는",
        correct: "책은",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "grammar.particle.subject-allomorph",
        title: "주격 조사 ‘이/가’",
        description: "앞말의 받침에 맞춰 주격 조사 ‘이/가’를 선택합니다.",
        incorrect: "나무이",
        correct: "나무가",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "grammar.particle.object-allomorph",
        title: "목적격 조사 ‘을/를’",
        description: "앞말의 받침에 맞춰 목적격 조사 ‘을/를’을 선택합니다.",
        incorrect: "책를",
        correct: "책을",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "grammar.particle.comitative-allomorph",
        title: "접속 조사 ‘과/와’",
        description: "앞말의 받침에 맞춰 접속 조사 ‘과/와’를 선택합니다.",
        incorrect: "책와",
        correct: "책과",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "grammar.particle.instrumental-allomorph",
        title: "부사격 조사 ‘으로/로’",
        description: "앞말의 받침에 맞춰 부사격 조사 ‘으로/로’를 선택합니다.",
        incorrect: "책로",
        correct: "책으로",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "punctuation.space-after-comma",
        title: "쉼표 뒤 띄어쓰기",
        description: "쉼표 뒤에 이어지는 한국어 문장을 한 칸 띄웁니다.",
        incorrect: "사과,배",
        correct: "사과, 배",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "punctuation.space-after-sentence-mark",
        title: "문장 부호 뒤 띄어쓰기",
        description: "마침표·느낌표·물음표 뒤의 다음 문장을 한 칸 띄웁니다.",
        incorrect: "끝났다.다음",
        correct: "끝났다. 다음",
        confidence: Confidence::High,
        default_enabled: true,
    },
    NativeRuleMetadata {
        id: "repetition.adjacent-word",
        title: "인접 단어 반복",
        description: "같은 단어가 바로 이어서 반복된 부분을 찾습니다.",
        incorrect: "문서를 문서를",
        correct: "문서를",
        confidence: Confidence::Medium,
        default_enabled: true,
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

fn native_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = native_literal_diagnostics(text, source_ranges, config);
    diagnostics.extend(parallel_deun_diagnostics(text, source_ranges, config));
    diagnostics.extend(particle_diagnostics(text, source_ranges, config));
    diagnostics.extend(punctuation_diagnostics(text, source_ranges, config));
    diagnostics.extend(repeated_word_diagnostics(text, source_ranges, config));
    diagnostics
}

fn native_literal_diagnostics(
    text: &str,
    source_ranges: &[TextRange],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    NATIVE_LITERALS
        .iter()
        .filter(|rule| !config.is_disabled(rule.id))
        .flat_map(|rule| {
            source_ranges.iter().flat_map(move |source_range| {
                text[source_range.start..source_range.end]
                    .match_indices(rule.from)
                    .map(move |(relative_start, original)| {
                        native_diagnostic(
                            rule.id,
                            rule.severity,
                            rule.message,
                            TextRange {
                                start: source_range.start + relative_start,
                                end: source_range.start + relative_start + original.len(),
                            },
                            original,
                            rule.to,
                            rule.safe_fix,
                        )
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
    if !config.is_disabled("grammar.particle.subject-allomorph") {
        for source_range in source_ranges {
            for (relative_start, _) in
                text[source_range.start..source_range.end].match_indices('이')
            {
                let start = source_range.start + relative_start;
                let end = start + '이'.len_utf8();
                let Some((previous_start, previous)) = text[..start].char_indices().last() else {
                    continue;
                };
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
                        &format!("{previous}가"),
                        true,
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
            let Some((previous_start, previous)) = text[..start].char_indices().last() else {
                continue;
            };
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
                    &format!("{previous}{}", rule.correct_particle),
                    true,
                ));
            }
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
    matches!(
        word,
        "책" | "문서" | "파일" | "값" | "글" | "댓글" | "사용자" | "프로젝트" | "코드"
    )
}

fn is_likely_subject_noun(word: &str) -> bool {
    matches!(
        word,
        "나무" | "사과" | "바다" | "학교" | "의자" | "모자" | "친구"
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
        for (relative_start, character) in source.char_indices() {
            let start = source_range.start + relative_start;
            let end = start + character.len_utf8();
            let next = text[end..].chars().next();
            if !config.is_disabled("punctuation.space-after-comma")
                && character == ','
                && next.is_some_and(is_hangul_syllable)
            {
                diagnostics.push(native_diagnostic(
                    "punctuation.space-after-comma",
                    Severity::Warning,
                    "쉼표 뒤에는 띄어쓰기를 넣으세요.",
                    TextRange { start, end },
                    &text[start..end],
                    ", ",
                    true,
                ));
            }
            if !config.is_disabled("punctuation.space-after-sentence-mark")
                && matches!(character, '.' | '!' | '?')
                && next.is_some_and(is_hangul_syllable)
                && text[..start]
                    .chars()
                    .last()
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

fn source_ranges(text: &str, source_kind: SourceKind) -> Vec<TextRange> {
    match source_kind {
        SourceKind::PlainText => vec![TextRange {
            start: 0,
            end: text.len(),
        }],
        SourceKind::Markdown => markdown_ranges(text),
        SourceKind::JavaScript | SourceKind::TypeScript | SourceKind::Python | SourceKind::Rust => {
            comment_ranges(text, source_kind)
        }
    }
}

fn markdown_ranges(text: &str) -> Vec<TextRange> {
    let mut ranges = Vec::new();
    let mut line_start = 0;
    let mut in_fence = false;

    for line in text.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let content = &text[line_start..line_end];
        let trimmed = content.trim_start_matches([' ', '\t']);
        let is_fence_marker = trimmed.starts_with("```") || trimmed.starts_with("~~~");

        if is_fence_marker {
            in_fence = !in_fence;
        } else if !in_fence {
            push_non_code_markdown_ranges(content, line_start, &mut ranges);
        }

        line_start = line_end;
    }

    ranges
}

fn push_non_code_markdown_ranges(line: &str, line_start: usize, ranges: &mut Vec<TextRange>) {
    let mut segment_start = line_start;
    let mut in_inline_code = false;

    for (relative_index, character) in line.char_indices() {
        if character != '`' {
            continue;
        }

        let current = line_start + relative_index;
        if in_inline_code {
            segment_start = current + character.len_utf8();
            in_inline_code = false;
        } else {
            push_non_empty_range(ranges, segment_start, current);
            in_inline_code = true;
        }
    }

    if !in_inline_code {
        push_non_empty_range(ranges, segment_start, line_start + line.len());
    }
}

#[cfg(feature = "source-parsing")]
fn comment_ranges(text: &str, source_kind: SourceKind) -> Vec<TextRange> {
    let language: tree_sitter::Language = match source_kind {
        SourceKind::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        SourceKind::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        SourceKind::Python => tree_sitter_python::LANGUAGE.into(),
        SourceKind::Rust => tree_sitter_rust::LANGUAGE.into(),
        SourceKind::PlainText | SourceKind::Markdown => return Vec::new(),
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(text, None) else {
        return Vec::new();
    };

    let mut ranges = Vec::new();
    collect_comment_ranges(tree.root_node(), &mut ranges);
    ranges
}

#[cfg(not(feature = "source-parsing"))]
fn comment_ranges(text: &str, source_kind: SourceKind) -> Vec<TextRange> {
    lightweight_comment_ranges(text, source_kind)
}

/// A dependency-free fallback for browser builds, where native Tree-sitter parsers
/// cannot be linked. It recognizes line and block comments while skipping quoted text.
#[cfg(not(feature = "source-parsing"))]
fn lightweight_comment_ranges(text: &str, source_kind: SourceKind) -> Vec<TextRange> {
    if source_kind == SourceKind::Python {
        return text
            .split_inclusive('\n')
            .scan(0, |line_start, line| {
                let start = *line_start;
                *line_start += line.len();
                Some((start, line))
            })
            .filter_map(|(line_start, line)| {
                line.find('#').map(|relative_start| TextRange {
                    start: line_start + relative_start,
                    end: line_start + line.trim_end_matches('\n').len(),
                })
            })
            .collect();
    }

    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;
    let mut quote = None;
    while index < bytes.len() {
        if let Some(delimiter) = quote {
            if bytes[index] == b'\\' {
                index += 2;
            } else if bytes[index] == delimiter {
                quote = None;
                index += 1;
            } else {
                index += 1;
            }
            continue;
        }

        match bytes[index] {
            b'\'' | b'\"' | b'`' => {
                quote = Some(bytes[index]);
                index += 1;
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let start = index;
                index = bytes[index..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |relative_end| index + relative_end);
                ranges.push(TextRange { start, end: index });
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let start = index;
                let content_start = index + 2;
                index = bytes[content_start..]
                    .windows(2)
                    .position(|window| window == b"*/")
                    .map_or(bytes.len(), |relative_end| content_start + relative_end + 2);
                ranges.push(TextRange { start, end: index });
            }
            _ => index += 1,
        }
    }
    ranges
}

#[cfg(feature = "source-parsing")]
fn collect_comment_ranges(node: tree_sitter::Node<'_>, ranges: &mut Vec<TextRange>) {
    if node.kind() == "comment" {
        ranges.push(TextRange {
            start: node.start_byte(),
            end: node.end_byte(),
        });
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_comment_ranges(child, ranges);
    }
}

fn push_non_empty_range(ranges: &mut Vec<TextRange>, start: usize, end: usize) {
    if start < end {
        ranges.push(TextRange { start, end });
    }
}
