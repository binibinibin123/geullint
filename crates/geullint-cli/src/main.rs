use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
#[cfg(feature = "standard")]
use geullint_core::{
    ContextRanker, DiagnosticV2, FixSafety, GeulRankSmall, StandardLexicon, StandardPipeline,
};
use geullint_core::{
    Diagnostic, DictionaryOverlay, Engine, LintConfig, Profile, RulePack, Severity, SourceKind,
    TextRange, rule_catalog, rule_metadata,
};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};
use std::thread;
use std::time::Duration;

mod cache;
mod evaluation_v2;
use evaluation_v2::{
    CaseMetadata, CorpusAnnotationOrigin, CorpusAnnotationStatus, CorpusOrigin, CorpusSplit,
    CorpusTextOrigin, DatasetMetadata, DatasetQualityGate, aggregate_metadata,
    evaluate_dataset_gate,
};

#[derive(Clone, Debug, Parser)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "geullint",
    about = "완전 오프라인 한국어 맞춤법·문법 린터",
    version,
    after_help = "LSP 서버: geullint lsp --stdio"
)]
struct Arguments {
    /// 검사할 파일 또는 디렉터리입니다.
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,

    /// 출력 형식입니다.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    /// 검사 실행 경로입니다. `standard`는 후보를 Review로 표시하고, `context`는 학습형
    /// 문맥 랭커를 실험적으로 사용하지만 모든 후보를 계속 Review로 표시합니다.
    #[arg(long, value_enum, default_value_t = EngineMode::Compact)]
    engine: EngineMode,

    /// JSON Lines gold corpus를 평가하고 JSON 정밀도·재현율 보고서를 출력합니다.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with_all = ["explain", "fix", "fix_dry_run"]
    )]
    corpus: Option<PathBuf>,

    /// 라이선스와 SHA-256이 기록된 corpus manifest를 검증하고 평가합니다.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with_all = ["corpus", "explain", "fix", "fix_dry_run"]
    )]
    corpus_manifest: Option<PathBuf>,

    /// corpus 보고서의 품질 임계값을 담은 로컬 JSON 파일입니다.
    #[arg(long, value_name = "PATH")]
    corpus_gate: Option<PathBuf>,

    /// 설정 JSON 파일 경로입니다. 지정하지 않으면 현재 디렉터리의 .geullint.json을 찾습니다.
    #[arg(long)]
    config: Option<PathBuf>,

    /// `geullint-overlay-v1` 형식의 프로젝트 사전 파일입니다. 여러 번 지정할 수 있습니다.
    #[arg(long = "dictionary-overlay", value_name = "PATH")]
    dictionary_overlays: Vec<PathBuf>,

    /// `version: 1` YAML 규칙 묶음입니다. 여러 번 지정할 수 있습니다.
    #[arg(long = "rule-pack", value_name = "PATH", conflicts_with = "explain")]
    rule_packs: Vec<PathBuf>,

    /// 이번 실행에서 비활성화할 규칙 ID입니다. 여러 번 지정할 수 있습니다.
    #[arg(long = "disable", value_name = "RULE_ID")]
    disabled_rules: Vec<String>,

    /// 규칙의 프로필·신뢰도·자동수정 안전도를 표시합니다.
    #[arg(long, value_name = "RULE_ID", conflicts_with_all = ["fix", "fix_dry_run"])]
    explain: Option<String>,

    /// 검사 규칙 묶음입니다.
    #[arg(long, value_enum)]
    profile: Option<ProfileArgument>,

    /// 이 심각도 이상 진단이 있으면 1로 종료합니다.
    #[arg(long, value_enum, default_value_t = FailOn::Error)]
    fail_on: FailOn,

    /// 안전한 수정만 원본 파일에 적용한 뒤 남은 진단을 출력합니다.
    #[arg(long)]
    fix: bool,

    /// 안전한 수정을 가상 적용하고 남는 진단만 출력합니다.
    #[arg(long, conflicts_with = "fix")]
    fix_dry_run: bool,

    /// 표준 입력의 한 문서만 검사합니다.
    #[arg(long, conflicts_with_all = ["changed", "watch", "corpus", "corpus_manifest"])]
    stdin: bool,

    /// Git의 staged·working tree·untracked 변경 중 지원 파일만 검사합니다.
    #[arg(long, conflicts_with_all = ["stdin", "watch", "corpus", "corpus_manifest"])]
    changed: bool,

    /// 파일이 바뀔 때마다 다시 검사합니다. Ctrl+C로 종료합니다.
    #[arg(long, conflicts_with_all = ["stdin", "changed", "corpus", "corpus_manifest"])]
    watch: bool,

    /// `--fix` 결과를 파일에 쓰지 않고 간단한 diff로 출력합니다.
    #[arg(long, requires = "fix", conflicts_with = "fix_dry_run")]
    diff: bool,

    /// 내용 해시를 `.geullint/cache-v1.json`에 저장해 재검사 시간을 줄입니다.
    #[arg(long, conflicts_with_all = ["stdin", "corpus", "corpus_manifest"])]
    cache: bool,

    /// ANSI 색상을 사용하지 않는 평문 출력을 고정합니다.
    ///
    /// `GeulLint`의 기본 출력도 평문이지만, 이 플래그를 CI·스크립트에 명시하면
    /// 출력 계약이 색상 코드에 의존하지 않는다는 의도를 드러낼 수 있습니다.
    #[arg(long)]
    no_color: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Sarif,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EngineMode {
    Compact,
    Standard,
    /// Experimental learned context ranking; all generated candidates remain Review-only.
    Context,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FailOn {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProfileArgument {
    Default,
    Strict,
    Editorial,
}

#[derive(Debug, Parser)]
#[command(
    name = "geullint rules",
    about = "번들된 검수 규칙의 메타데이터를 출력합니다"
)]
struct RulesArguments {
    /// 규칙 카탈로그 출력 형식입니다.
    #[arg(long, value_enum, default_value_t = CatalogFormat::Json)]
    format: CatalogFormat,
}

#[derive(Debug, Parser)]
#[command(name = "geullint init", about = "프로젝트 설정 파일을 생성합니다")]
struct InitArguments {
    /// 설정 파일을 만들 프로젝트 경로입니다.
    #[arg(value_name = "PATH", default_value = ".")]
    path: PathBuf,
    /// 이미 있는 파일도 기본 설정으로 덮어씁니다.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Parser)]
#[command(name = "geullint doctor", about = "로컬 설정과 사전 상태를 점검합니다")]
struct DoctorArguments {
    /// 점검할 설정 파일입니다.
    #[arg(long)]
    config: Option<PathBuf>,
    /// 출력 형식입니다.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
#[command(name = "geullint dictionary", about = "프로젝트 사전을 관리합니다")]
struct DictionaryArguments {
    #[command(subcommand)]
    command: DictionaryCommand,
}

#[derive(Debug, Parser)]
#[command(name = "geullint feedback", about = "로컬 피드백을 내보냅니다")]
struct FeedbackArguments {
    #[command(subcommand)]
    command: FeedbackCommand,
}

#[derive(Debug, Subcommand)]
enum FeedbackCommand {
    /// 로컬 JSONL 피드백을 개인정보 필드 없이 복사합니다.
    Export {
        /// 입력 JSONL. 기본값은 `.geullint/feedback.jsonl`입니다.
        #[arg(long)]
        input: Option<PathBuf>,
        /// 출력 JSONL. 기본값은 `geullint-feedback.jsonl`입니다.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Parser)]
#[command(
    name = "geullint completion",
    about = "셸 자동 완성 스크립트를 출력합니다"
)]
struct CompletionArguments {
    #[arg(value_enum)]
    shell: CompletionShell,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

#[derive(Debug, Subcommand)]
enum DictionaryCommand {
    /// overlay 파일을 파싱하고 항목 수를 확인합니다.
    Validate {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum CatalogFormat {
    #[default]
    Json,
    Markdown,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleCatalog {
    version: u8,
    rule_count: usize,
    rules: Vec<geullint_core::RuleMetadata>,
}

impl From<ProfileArgument> for Profile {
    fn from(value: ProfileArgument) -> Self {
        match value {
            ProfileArgument::Default => Self::Default,
            ProfileArgument::Strict => Self::Strict,
            ProfileArgument::Editorial => Self::Editorial,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonReport {
    version: u8,
    diagnostics: Vec<ReportedDiagnostic>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportedDiagnostic {
    path: String,
    line: usize,
    column: usize,
    #[serde(flatten)]
    diagnostic: Diagnostic,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLog {
    version: &'static str,
    #[serde(rename = "$schema")]
    schema: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRun {
    tool: SarifTool,
    column_kind: &'static str,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifDriver {
    name: &'static str,
    information_uri: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: usize,
    start_column: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CorpusCase {
    id: String,
    text: String,
    #[serde(default)]
    genre: Option<String>,
    #[serde(default)]
    source_kind: SourceKind,
    #[serde(default)]
    profile: Option<Profile>,
    #[serde(default)]
    case_type: Option<CorpusCaseType>,
    #[serde(default)]
    provenance_id: Option<String>,
    #[serde(default)]
    origin: CorpusOrigin,
    #[serde(default)]
    text_origin: Option<CorpusTextOrigin>,
    #[serde(default)]
    annotation_origin: Option<CorpusAnnotationOrigin>,
    #[serde(default)]
    annotation_status: Option<CorpusAnnotationStatus>,
    #[serde(default)]
    split: Option<CorpusSplit>,
    #[serde(default)]
    holdout_id: Option<String>,
    #[serde(default)]
    document_id: Option<String>,
    #[serde(default)]
    author_id: Option<String>,
    #[serde(default)]
    source_id: Option<String>,
    #[serde(default)]
    source_sha256: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    review_provenance: Option<CorpusReviewProvenance>,
    #[serde(default)]
    error_families: Vec<String>,
    #[serde(default)]
    expected_rule_ids: Vec<String>,
    #[serde(default)]
    expected_diagnostics: Vec<CorpusExpectedDiagnostic>,
    #[serde(default)]
    expected_fixed_text: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CorpusReviewProvenance {
    #[serde(default)]
    reviewer_type: Option<String>,
    #[serde(default)]
    adjudicator_type: Option<String>,
    #[serde(default)]
    adjudicator_id: Option<String>,
    #[serde(default)]
    human_evidence: Option<serde_json::Value>,
    #[serde(default)]
    model_snapshots: Vec<String>,
    rubric_sha256: Option<String>,
    session_sha256: Option<String>,
    output_sha256: Option<String>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CorpusCaseType {
    Error,
    Normal,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CorpusExpectedDiagnostic {
    rule_id: String,
    #[serde(default)]
    original: Option<String>,
    #[serde(default)]
    range: Option<TextRange>,
    #[serde(default)]
    suggestions: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CorpusManifest {
    schema_version: u8,
    name: String,
    license: String,
    source_url: String,
    corpus_path: PathBuf,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CorpusQualityGate {
    schema_version: u8,
    min_micro_precision: f64,
    min_macro_precision: f64,
    min_recall: f64,
    #[serde(default)]
    min_top1_correction_accuracy: Option<f64>,
    #[serde(default)]
    min_top5_correction_accuracy: Option<f64>,
    min_rule_precision_wilson_lower_95: f64,
    min_expected_per_rule: usize,
    required_rule_ids: Vec<String>,
    #[serde(default)]
    dataset: DatasetQualityGate,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusProvenance {
    name: String,
    license: String,
    source_url: String,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusCaseFailure {
    id: String,
    false_positive_rule_ids: Vec<String>,
    false_negative_rule_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fixed_text_mismatch: Option<CorpusFixedTextMismatch>,
}

#[derive(Serialize)]
struct CorpusFixedTextMismatch {
    expected: String,
    actual: String,
}

#[derive(Default)]
struct RuleMetricCounts {
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleCorpusMetric {
    rule_id: String,
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
    precision: Option<f64>,
    recall: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    precision_wilson_lower_95: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusQualityGateFailure {
    metric: &'static str,
    actual: Option<f64>,
    minimum: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    holdout_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusQualityGateReport {
    passed: bool,
    failures: Vec<CorpusQualityGateFailure>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusReport {
    version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<CorpusProvenance>,
    cases: usize,
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
    precision: Option<f64>,
    recall: Option<f64>,
    macro_precision: Option<f64>,
    macro_recall: Option<f64>,
    normal_cases: usize,
    false_positive_cases: usize,
    specificity: Option<f64>,
    correction_cases: usize,
    top1_correction_accuracy: Option<f64>,
    top5_correction_accuracy: Option<f64>,
    dataset: DatasetMetadata,
    rule_metrics: Vec<RuleCorpusMetric>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality_gate: Option<CorpusQualityGateReport>,
    case_failures: Vec<CorpusCaseFailure>,
}

fn main() -> ExitCode {
    let raw_arguments: Vec<String> = std::env::args().collect();
    if raw_arguments.get(1).map(String::as_str) == Some("lsp") {
        return run_lsp();
    }
    if raw_arguments.get(1).map(String::as_str) == Some("rules") {
        return run_rules();
    }
    if raw_arguments.get(1).map(String::as_str) == Some("init") {
        return run_init(&raw_arguments);
    }
    if raw_arguments.get(1).map(String::as_str) == Some("doctor") {
        return run_doctor(&raw_arguments);
    }
    if raw_arguments.get(1).map(String::as_str) == Some("dictionary") {
        return run_dictionary(&raw_arguments);
    }
    if raw_arguments.get(1).map(String::as_str) == Some("feedback") {
        return run_feedback(&raw_arguments);
    }
    if raw_arguments.get(1).map(String::as_str) == Some("completion") {
        return run_completion(&raw_arguments);
    }

    let normalized_arguments = normalize_alias(raw_arguments);
    let arguments = match Arguments::try_parse_from(normalized_arguments) {
        Ok(arguments) => arguments,
        Err(error) => {
            let exit_code = u8::try_from(error.exit_code()).unwrap_or(2);
            let _ = error.print();
            return ExitCode::from(exit_code);
        }
    };
    match run(&arguments) {
        Ok(has_failure) if has_failure => ExitCode::from(1),
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("geullint: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn normalize_alias(mut arguments: Vec<String>) -> Vec<String> {
    match arguments.get(1).map(String::as_str) {
        Some("check") => {
            arguments.remove(1);
        }
        Some("fix") => {
            arguments.remove(1);
            if !arguments.iter().any(|argument| argument == "--fix") {
                arguments.push("--fix".to_owned());
            }
        }
        _ => {}
    }
    arguments
}

fn parse_subcommand<T>(arguments: &[String], name: &str) -> Result<T>
where
    T: Parser,
{
    T::try_parse_from(
        std::iter::once(format!("geullint {name}")).chain(arguments.iter().skip(2).cloned()),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn run_init(raw_arguments: &[String]) -> ExitCode {
    let arguments = match parse_subcommand::<InitArguments>(raw_arguments, "init") {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("geullint init: {error:#}");
            return ExitCode::from(2);
        }
    };
    match initialize_project(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("geullint init: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run_doctor(raw_arguments: &[String]) -> ExitCode {
    let arguments = match parse_subcommand::<DoctorArguments>(raw_arguments, "doctor") {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("geullint doctor: {error:#}");
            return ExitCode::from(2);
        }
    };
    match diagnose_project(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("geullint doctor: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run_dictionary(raw_arguments: &[String]) -> ExitCode {
    let arguments = match parse_subcommand::<DictionaryArguments>(raw_arguments, "dictionary") {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("geullint dictionary: {error:#}");
            return ExitCode::from(2);
        }
    };
    match validate_dictionary(arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("geullint dictionary: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run_feedback(raw_arguments: &[String]) -> ExitCode {
    let arguments = match parse_subcommand::<FeedbackArguments>(raw_arguments, "feedback") {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("geullint feedback: {error:#}");
            return ExitCode::from(2);
        }
    };
    match export_feedback(arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("geullint feedback: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run_completion(raw_arguments: &[String]) -> ExitCode {
    let arguments = match parse_subcommand::<CompletionArguments>(raw_arguments, "completion") {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("geullint completion: {error:#}");
            return ExitCode::from(2);
        }
    };
    print_completion(arguments.shell);
    ExitCode::SUCCESS
}

fn run_rules() -> ExitCode {
    let arguments = RulesArguments::try_parse_from(
        std::iter::once("geullint rules".to_owned()).chain(std::env::args().skip(2)),
    );
    let arguments = match arguments {
        Ok(arguments) => arguments,
        Err(error) => {
            let exit_code = u8::try_from(error.exit_code()).unwrap_or(2);
            let _ = error.print();
            return ExitCode::from(exit_code);
        }
    };
    match print_rule_catalog(arguments.format) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("geullint rules: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn bundled_rule_catalog() -> RuleCatalog {
    let rules = rule_catalog();
    RuleCatalog {
        version: 1,
        rule_count: rules.len(),
        rules,
    }
}

fn print_rule_catalog(format: CatalogFormat) -> Result<()> {
    let catalog = bundled_rule_catalog();
    match format {
        CatalogFormat::Json => println!("{}", serde_json::to_string_pretty(&catalog)?),
        CatalogFormat::Markdown => {
            let mut markdown = format!("# GeulLint 규칙 {}개\n\n", catalog.rule_count);
            writeln!(
                markdown,
                "> 이 문서는 `geullint rules --format markdown`으로 생성됩니다.\n"
            )?;
            for rule in catalog.rules {
                writeln!(markdown, "<a id=\"{}\"></a>", rule.id)?;
                writeln!(markdown, "## `{}` — {}\n", rule.id, rule.title)?;
                writeln!(markdown, "{}\n", rule.description)?;
                writeln!(
                    markdown,
                    "- 분류: `{}`\n- 신뢰도: `{:?}`\n- 자동 수정: `{:?}`\n- 기본 활성화: `{}`\n",
                    rule.category, rule.confidence, rule.fix_safety, rule.default_enabled
                )?;
                if let (Some(incorrect), Some(correct)) = (
                    rule.incorrect_examples.first(),
                    rule.correct_examples.first(),
                ) {
                    writeln!(
                        markdown,
                        "- 예: `{}` → `{}`\n",
                        incorrect.replace('`', "\\`"),
                        correct.replace('`', "\\`")
                    )?;
                }
            }
            print!("{markdown}");
        }
    }
    Ok(())
}

fn run_lsp() -> ExitCode {
    let extra_arguments: Vec<_> = std::env::args().skip(2).collect();
    if extra_arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        println!("Usage: geullint lsp --stdio\n\n표준 입출력으로 GeulLint LSP 서버를 시작합니다.");
        return ExitCode::SUCCESS;
    }
    if extra_arguments.iter().any(|argument| argument != "--stdio") {
        eprintln!("geullint lsp: 지원하는 옵션은 --stdio뿐입니다.");
        return ExitCode::from(2);
    }
    match tokio::runtime::Runtime::new() {
        Ok(runtime) => {
            runtime.block_on(geullint_lsp::run_stdio());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("geullint lsp: Tokio 런타임을 시작할 수 없습니다: {error}");
            ExitCode::from(2)
        }
    }
}

fn initialize_project(arguments: &InitArguments) -> Result<()> {
    let project_path = if arguments.path.is_file() {
        bail!("{} 경로는 디렉터리가 아닙니다", arguments.path.display());
    } else {
        &arguments.path
    };
    fs::create_dir_all(project_path)
        .with_context(|| format!("{} 디렉터리를 만들 수 없습니다", project_path.display()))?;
    let config_path = project_path.join(".geullint.json");
    if config_path.exists() && !arguments.force {
        bail!(
            "{} 파일이 이미 있습니다 (--force로 덮어쓸 수 있습니다)",
            config_path.display()
        );
    }
    let config = serde_json::to_string_pretty(&LintConfig::default())? + "\n";
    fs::write(&config_path, config)
        .with_context(|| format!("{} 파일을 쓸 수 없습니다", config_path.display()))?;
    let ignore_path = project_path.join(".geullintignore");
    if !ignore_path.exists() {
        fs::write(&ignore_path, "target/\nnode_modules/\n.next/\n")
            .with_context(|| format!("{} 파일을 쓸 수 없습니다", ignore_path.display()))?;
    }
    println!(
        "GeulLint 프로젝트를 초기화했습니다: {}",
        project_path.display()
    );
    println!("  설정: {}", config_path.display());
    Ok(())
}

fn diagnose_project(arguments: &DoctorArguments) -> Result<()> {
    let config_path = arguments
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from(".geullint.json"));
    let config_exists = config_path.is_file();
    let (config_valid, dictionary_entries, config_error) = if config_exists {
        match fs::read_to_string(&config_path)
            .with_context(|| format!("{} 파일을 읽을 수 없습니다", config_path.display()))
            .and_then(|contents| {
                serde_json::from_str::<LintConfig>(&contents).with_context(|| {
                    format!("{} 설정 JSON이 올바르지 않습니다", config_path.display())
                })
            }) {
            Ok(config) => (
                true,
                config.user_dictionary.len() + config.dictionary_overlay.len(),
                None,
            ),
            Err(error) => (false, 0, Some(error.to_string())),
        }
    } else {
        (false, 0, Some("설정 파일이 없습니다".to_owned()))
    };
    let status = if config_valid { "ok" } else { "warning" };
    match arguments.format {
        OutputFormat::Json => {
            let report = serde_json::json!({
                "version": 1,
                "status": status,
                "configuration": {
                    "path": config_path,
                    "exists": config_exists,
                    "valid": config_valid,
                    "error": config_error,
                },
                "dictionary": { "entries": dictionary_entries },
            });
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        OutputFormat::Human => {
            println!("GeulLint doctor: {status}");
            println!("  configuration: {}", config_path.display());
            println!("  exists: {config_exists}");
            println!("  valid: {config_valid}");
            println!("  dictionary entries: {dictionary_entries}");
            if let Some(error) = config_error {
                println!("  note: {error}");
            }
        }
        OutputFormat::Sarif => bail!("doctor에는 --format human 또는 json을 사용하세요"),
    }
    Ok(())
}

fn validate_dictionary(arguments: DictionaryArguments) -> Result<()> {
    match arguments.command {
        DictionaryCommand::Validate { path } => {
            let source = fs::read_to_string(&path)
                .with_context(|| format!("{} 파일을 읽을 수 없습니다", path.display()))?;
            let overlay = DictionaryOverlay::parse(&source)
                .with_context(|| format!("{} overlay 형식이 올바르지 않습니다", path.display()))?;
            println!(
                "{}: valid geullint-overlay-v1 ({} entries)",
                path.display(),
                overlay.entry_count()
            );
        }
    }
    Ok(())
}

fn export_feedback(arguments: FeedbackArguments) -> Result<()> {
    match arguments.command {
        FeedbackCommand::Export { input, output } => {
            let input = input.unwrap_or_else(|| PathBuf::from(".geullint/feedback.jsonl"));
            let output = output.unwrap_or_else(|| PathBuf::from("geullint-feedback.jsonl"));
            let source = if input.is_file() {
                fs::read_to_string(&input)
                    .with_context(|| format!("{} 피드백을 읽을 수 없습니다", input.display()))?
            } else {
                String::new()
            };
            let mut exported = Vec::new();
            for line in source.lines() {
                let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                let Some(object) = value.as_object() else {
                    continue;
                };
                let mut sanitized = serde_json::Map::new();
                for key in [
                    "version",
                    "ruleId",
                    "accepted",
                    "replacement",
                    "sourceKind",
                    "profile",
                ] {
                    if let Some(value) = object.get(key) {
                        sanitized.insert(key.to_owned(), value.clone());
                    }
                }
                if sanitized.contains_key("ruleId") {
                    exported.push(serde_json::Value::Object(sanitized));
                }
            }
            let mut serialized = String::new();
            for value in &exported {
                writeln!(serialized, "{}", serde_json::to_string(value)?)?;
            }
            cache::write_atomic_text(&output, &serialized)?;
            println!(
                "로컬 피드백 {}건을 {}에 내보냈습니다 (네트워크 전송 없음)",
                exported.len(),
                output.display()
            );
        }
    }
    Ok(())
}

fn print_completion(shell: CompletionShell) {
    let script = match shell {
        CompletionShell::Bash => {
            r#"_geullint_complete() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=( $(compgen -W "check fix init doctor dictionary feedback completion rules lsp --stdin --changed --watch --fix --format --engine --no-color" -- "$cur") )
}
complete -F _geullint_complete geullint
"#
        }
        CompletionShell::Zsh => {
            r"#compdef geullint
_arguments '1:command:(check fix init doctor dictionary feedback completion rules lsp)' '*:path:_files'
"
        }
        CompletionShell::Fish => {
            r"complete -c geullint -f -n '__fish_use_subcommand' -a 'check fix init doctor dictionary feedback completion rules lsp'
complete -c geullint -l stdin -d 'read one document from stdin'
complete -c geullint -l changed -d 'check changed files'
complete -c geullint -l fix -d 'apply safe fixes'
complete -c geullint -l engine -a 'compact standard context' -d 'select lint engine'
complete -c geullint -l no-color -d 'disable ANSI color output'
"
        }
        CompletionShell::Powershell => {
            r#"Register-ArgumentCompleter -Native -CommandName geullint -ScriptBlock {
  param($wordToComplete, $commandAst, $cursorPosition)
  'check','fix','init','doctor','dictionary','feedback','completion','rules','lsp','--stdin','--changed','--watch','--fix','--format','--engine','--no-color' |
    Where-Object { $_ -like "$wordToComplete*" } |
    ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_) }
}
"#
        }
    };
    print!("{script}");
}

fn run(arguments: &Arguments) -> Result<bool> {
    if arguments.watch {
        return run_watch(arguments);
    }
    run_once(arguments)
}

fn run_watch(arguments: &Arguments) -> Result<bool> {
    let mut first_run = true;
    loop {
        let mut one_shot = arguments.clone();
        one_shot.watch = false;
        let has_failure = run_once(&one_shot)?;
        if first_run {
            eprintln!("geullint: watching for changes (Ctrl+C to stop)");
            first_run = false;
        }
        thread::sleep(Duration::from_millis(700));
        if has_failure {
            // Keep watching after a failed lint; editors commonly save transiently invalid text.
        }
    }
}

#[allow(clippy::too_many_lines)]
fn run_once(arguments: &Arguments) -> Result<bool> {
    if let Some(rule_id) = &arguments.explain {
        print_rule_explanation(rule_id, arguments.format)?;
        return Ok(false);
    }
    let config = load_config(arguments)?;
    let packs = load_rule_packs(arguments)?;
    let quality_gate = arguments
        .corpus_gate
        .as_deref()
        .map(load_corpus_quality_gate)
        .transpose()?;
    if let Some(corpus_path) = &arguments.corpus {
        return evaluate_corpus(corpus_path, &config, None, &packs, quality_gate.as_ref());
    }
    if let Some(manifest_path) = &arguments.corpus_manifest {
        let (corpus_path, provenance) = load_corpus_manifest(manifest_path)?;
        return evaluate_corpus(
            &corpus_path,
            &config,
            Some(provenance),
            &packs,
            quality_gate.as_ref(),
        );
    }
    if quality_gate.is_some() {
        bail!("--corpus-gate에는 --corpus 또는 --corpus-manifest가 필요합니다");
    }

    if matches!(arguments.engine, EngineMode::Standard | EngineMode::Context) {
        #[cfg(feature = "standard")]
        {
            return run_standard_once(
                arguments,
                config,
                packs,
                matches!(arguments.engine, EngineMode::Context),
            );
        }
        #[cfg(not(feature = "standard"))]
        {
            let _ = (config, packs);
            bail!("이 CLI는 standard feature 없이 빌드되었습니다; all-features로 다시 빌드하세요");
        }
    }
    let engine = build_engine(config, packs)?;
    let mut reported = Vec::new();
    let cache_path = PathBuf::from(".geullint/cache-v1.json");
    let mut cache_file = arguments
        .cache
        .then(|| cache::load(&cache_path))
        .transpose()?;

    if arguments.stdin {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .context("표준 입력을 읽을 수 없습니다")?;
        append_diagnostics(
            &engine,
            "<stdin>",
            &text,
            SourceKind::PlainText,
            &mut reported,
        );
    } else {
        let paths = if arguments.changed {
            collect_changed_files()?
        } else {
            collect_files(&arguments.paths)?
        };
        for path in paths {
            let original = fs::read_to_string(&path)
                .with_context(|| format!("{} 파일을 UTF-8로 읽을 수 없습니다", path.display()))?;
            let source_kind = source_kind_for_path(&path);
            let mut text = original.clone();
            if arguments.fix || arguments.fix_dry_run {
                let fixed = engine
                    .check_with_fixes(&text, source_kind, false)
                    .fixed_text;
                if arguments.fix && !arguments.diff && fixed != text {
                    cache::write_atomic_text(&path, &fixed).with_context(|| {
                        format!("{} 파일에 수정 사항을 쓸 수 없습니다", path.display())
                    })?;
                }
                if arguments.diff && fixed != text {
                    print_diff(&path, &text, &fixed);
                }
                text = fixed;
            }
            let cache_key = path.display().to_string();
            let source_hash = cache::source_hash(&text);
            let cached = cache_file.as_ref().and_then(|file| {
                file.files
                    .get(&cache_key)
                    .filter(|entry| entry.source_hash == source_hash)
            });
            if let Some(entry) = cached {
                append_reported_diagnostics(&cache_key, &text, &entry.diagnostics, &mut reported);
            } else {
                let diagnostics = engine.check(&text, source_kind);
                if let Some(file) = cache_file.as_mut() {
                    file.files.insert(
                        cache_key.clone(),
                        cache::CacheEntry {
                            source_hash,
                            diagnostics: diagnostics.clone(),
                        },
                    );
                }
                append_reported_diagnostics(&cache_key, &text, &diagnostics, &mut reported);
            }
        }
    }

    let has_failure = reported.iter().any(|reported_diagnostic| {
        reaches_threshold(reported_diagnostic.diagnostic.severity, arguments.fail_on)
    });
    print_report(&reported, arguments.format, arguments.no_color)?;
    if let Some(cache_file) = cache_file {
        cache::save(&cache_path, &cache_file)?;
    }
    Ok(has_failure)
}

#[cfg(feature = "standard")]
fn run_standard_once(
    arguments: &Arguments,
    config: LintConfig,
    packs: Vec<RulePack>,
    use_context_ranker: bool,
) -> Result<bool> {
    if arguments.cache {
        bail!("--cache는 현재 compact 엔진에서만 지원합니다 (--engine compact)");
    }
    let engine = build_engine(config, packs)?;
    let lexicon = StandardLexicon::bundled()
        .map_err(|error| anyhow::anyhow!("표준 사전을 읽을 수 없습니다: {error}"))?;
    let ranker = GeulRankSmall::bundled()
        .map_err(|error| anyhow::anyhow!("standard ranker를 읽을 수 없습니다: {error}"))?;
    let pipeline = if use_context_ranker {
        let context_ranker = ContextRanker::bundled()
            .map_err(|error| anyhow::anyhow!("context ranker를 읽을 수 없습니다: {error}"))?;
        StandardPipeline::new(engine, lexicon, ranker).with_context_ranker(context_ranker)
    } else {
        StandardPipeline::new(engine, lexicon, ranker)
    };
    let mut reported = Vec::new();

    if arguments.stdin {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .context("표준 입력을 읽을 수 없습니다")?;
        let diagnostics = pipeline
            .check(&text, SourceKind::PlainText)
            .iter()
            .map(standard_diagnostic_to_legacy)
            .collect::<Vec<_>>();
        append_reported_diagnostics("<stdin>", &text, &diagnostics, &mut reported);
    } else {
        let paths = if arguments.changed {
            collect_changed_files()?
        } else {
            collect_files(&arguments.paths)?
        };
        for path in paths {
            let original = fs::read_to_string(&path)
                .with_context(|| format!("{} 파일을 UTF-8로 읽을 수 없습니다", path.display()))?;
            let source_kind = source_kind_for_path(&path);
            let mut text = original.clone();
            if arguments.fix || arguments.fix_dry_run {
                let fixed = pipeline
                    .check_with_fixes(&text, source_kind, false)
                    .fixed_text;
                if arguments.fix && !arguments.diff && fixed != text {
                    cache::write_atomic_text(&path, &fixed).with_context(|| {
                        format!("{} 파일에 수정 사항을 쓸 수 없습니다", path.display())
                    })?;
                }
                if arguments.diff && fixed != text {
                    print_diff(&path, &text, &fixed);
                }
                text = fixed;
            }
            let diagnostics = pipeline
                .check(&text, source_kind)
                .iter()
                .map(standard_diagnostic_to_legacy)
                .collect::<Vec<_>>();
            let cache_key = path.display().to_string();
            append_reported_diagnostics(&cache_key, &text, &diagnostics, &mut reported);
        }
    }

    let has_failure = reported.iter().any(|reported_diagnostic| {
        reaches_threshold(reported_diagnostic.diagnostic.severity, arguments.fail_on)
    });
    print_report(&reported, arguments.format, arguments.no_color)?;
    Ok(has_failure)
}

#[cfg(feature = "standard")]
fn standard_diagnostic_to_legacy(diagnostic: &DiagnosticV2) -> Diagnostic {
    Diagnostic {
        rule_id: diagnostic.rule_id.clone(),
        severity: diagnostic.severity,
        message: diagnostic.message.clone(),
        range: diagnostic.range,
        original: diagnostic.original.clone(),
        suggestions: diagnostic
            .suggestions
            .iter()
            .map(|suggestion| suggestion.text.clone())
            .collect(),
        safe_fix: diagnostic.safety == FixSafety::Safe,
    }
}

fn append_diagnostics(
    engine: &Engine,
    path: &str,
    text: &str,
    source_kind: SourceKind,
    reported: &mut Vec<ReportedDiagnostic>,
) {
    let diagnostics = engine.check(text, source_kind);
    append_reported_diagnostics(path, text, &diagnostics, reported);
}

fn append_reported_diagnostics(
    path: &str,
    text: &str,
    diagnostics: &[Diagnostic],
    reported: &mut Vec<ReportedDiagnostic>,
) {
    for diagnostic in diagnostics {
        let (line, column) = line_and_column(text, diagnostic.range.start);
        reported.push(ReportedDiagnostic {
            path: path.to_owned(),
            line,
            column,
            diagnostic: diagnostic.clone(),
        });
    }
}

fn collect_changed_files() -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    for arguments in [
        &["diff", "--name-only", "-z", "--diff-filter=ACMR"][..],
        &[
            "diff",
            "--cached",
            "--name-only",
            "-z",
            "--diff-filter=ACMR",
        ][..],
        &["ls-files", "--others", "--exclude-standard", "-z"][..],
    ] {
        candidates.extend(git_name_only_paths(arguments)?);
    }

    let mut paths = Vec::new();
    for path in candidates {
        if path.is_file() && supported_source_kind(&path).is_some() {
            paths.push(path);
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn git_name_only_paths(arguments: &[&str]) -> Result<Vec<PathBuf>> {
    let output = ProcessCommand::new("git")
        .args(arguments)
        .output()
        .with_context(|| format!("git {}를 실행할 수 없습니다", arguments.join(" ")))?;
    if !output.status.success() {
        bail!("git {}가 실패했습니다", arguments.join(" "));
    }
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| PathBuf::from(String::from_utf8_lossy(entry).into_owned()))
        .collect())
}

fn print_diff(path: &Path, original: &str, fixed: &str) {
    println!("--- {}", path.display());
    println!("+++ {} (geullint --fix)", path.display());
    for (line_number, (before, after)) in original.lines().zip(fixed.lines()).enumerate() {
        if before != after {
            println!("@@ line {} @@", line_number + 1);
            println!("-{before}");
            println!("+{after}");
        }
    }
    if original.lines().count() != fixed.lines().count() {
        println!("@@ line count changed @@");
    }
}

fn profile_engine_index(profile: Profile) -> usize {
    match profile {
        Profile::Default => 0,
        Profile::Strict => 1,
        Profile::Editorial => 2,
    }
}

fn validate_corpus_case_type(
    path: &Path,
    line: usize,
    case_type: Option<CorpusCaseType>,
    expected_diagnostic_count: usize,
) -> Result<()> {
    match case_type {
        Some(CorpusCaseType::Normal) if expected_diagnostic_count != 0 => bail!(
            "{} corpus line {line} normal caseType requires no expected diagnostics",
            path.display()
        ),
        Some(CorpusCaseType::Error) if expected_diagnostic_count == 0 => bail!(
            "{} corpus line {line} error caseType requires at least one expected diagnostic",
            path.display()
        ),
        _ => Ok(()),
    }
}

fn resolve_expected_diagnostic_ranges(
    path: &Path,
    line: usize,
    text: &str,
    expectations: &mut [CorpusExpectedDiagnostic],
) -> Result<()> {
    for expectation in expectations {
        if let Some(range) = expectation.range
            && (range.start > range.end
                || range.end > text.len()
                || !text.is_char_boundary(range.start)
                || !text.is_char_boundary(range.end))
        {
            bail!(
                "{} corpus line {line} expected diagnostic `{}` has an invalid UTF-8 range",
                path.display(),
                expectation.rule_id
            );
        }

        let Some(original) = expectation.original.as_deref() else {
            continue;
        };
        if original.is_empty() {
            bail!(
                "{} corpus line {line} expected diagnostic `{}` has an empty original",
                path.display(),
                expectation.rule_id
            );
        }
        if let Some(range) = expectation.range {
            if &text[range.start..range.end] != original {
                bail!(
                    "{} corpus line {line} expected diagnostic `{}` range does not equal original",
                    path.display(),
                    expectation.rule_id
                );
            }
            continue;
        }

        let mut occurrences = text
            .char_indices()
            .filter_map(|(start, _)| text[start..].starts_with(original).then_some(start));
        let first = occurrences.next();
        if first.is_none() || occurrences.next().is_some() {
            bail!(
                "{} corpus line {line} expected diagnostic `{}` original must occur exactly once",
                path.display(),
                expectation.rule_id
            );
        }
        let start = first.expect("a unique occurrence has a start");
        expectation.range = Some(TextRange {
            start,
            end: start + original.len(),
        });
    }
    Ok(())
}

fn is_sha256(value: Option<&String>) -> bool {
    value.is_some_and(|value| {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    })
}

#[allow(clippy::too_many_lines)]
fn validate_corpus_metadata(path: &Path, line: usize, case: &CorpusCase) -> Result<()> {
    let text_origin = case.text_origin.unwrap_or(match case.origin {
        CorpusOrigin::IndependentHuman => CorpusTextOrigin::HumanAuthored,
        CorpusOrigin::Revision => CorpusTextOrigin::Revision,
        CorpusOrigin::Project => CorpusTextOrigin::Project,
        CorpusOrigin::Synthetic => CorpusTextOrigin::Synthetic,
    });
    let requires_external_metadata =
        text_origin != CorpusTextOrigin::Project || case.text_origin.is_some();

    if case
        .source_id
        .as_deref()
        .is_some_and(|source_id| source_id.trim().is_empty())
    {
        bail!(
            "{} corpus line {line} sourceId cannot be empty",
            path.display()
        );
    }

    if let Some(split) = case.split {
        let is_holdout = matches!(split, CorpusSplit::H1 | CorpusSplit::H2);
        if is_holdout {
            if case.holdout_id.as_deref() != Some(split.as_str()) {
                bail!(
                    "{} corpus line {line} {} split requires holdoutId `{}`",
                    path.display(),
                    split.as_str(),
                    split.as_str()
                );
            }
        } else if case.holdout_id.is_some() {
            bail!(
                "{} corpus line {line} holdoutId is only valid for H1 or H2",
                path.display()
            );
        }
    } else if case.holdout_id.is_some() {
        bail!(
            "{} corpus line {line} holdoutId requires a split",
            path.display()
        );
    }

    if let Some(annotation_origin) = case.annotation_origin {
        let provenance = case.review_provenance.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "{} corpus line {line} annotationOrigin `{}` requires reviewProvenance",
                path.display(),
                serde_json::to_string(&annotation_origin).unwrap_or_else(|_| "unknown".to_owned())
            )
        })?;
        if provenance
            .model_snapshots
            .iter()
            .any(|snapshot| snapshot.trim().is_empty())
        {
            bail!(
                "{} corpus line {line} reviewProvenance modelSnapshots cannot contain empty values",
                path.display()
            );
        }
        if provenance
            .adjudicator_type
            .as_deref()
            .is_some_and(|kind| kind.trim().is_empty())
            || provenance
                .adjudicator_id
                .as_deref()
                .is_some_and(|id| id.trim().is_empty())
        {
            bail!(
                "{} corpus line {line} reviewProvenance adjudicator fields cannot be empty",
                path.display()
            );
        }
        for (field, value) in [
            ("rubricSha256", &provenance.rubric_sha256),
            ("sessionSha256", &provenance.session_sha256),
            ("outputSha256", &provenance.output_sha256),
        ] {
            if !is_sha256(value.as_ref()) {
                bail!(
                    "{} corpus line {line} reviewProvenance {field} must be 64 lowercase hexadecimal characters",
                    path.display()
                );
            }
        }
        match annotation_origin {
            CorpusAnnotationOrigin::AiBlindPanel => {
                if provenance.reviewer_type.as_deref() != Some("ai") {
                    bail!(
                        "{} corpus line {line} AI blind-panel annotation requires reviewerType `ai`",
                        path.display()
                    );
                }
                if provenance.adjudicator_type.as_deref() != Some("ai") {
                    bail!(
                        "{} corpus line {line} AI blind-panel annotation requires adjudicatorType `ai`",
                        path.display()
                    );
                }
                if provenance.model_snapshots.len() < 2 {
                    bail!(
                        "{} corpus line {line} AI blind-panel annotation requires at least two modelSnapshots",
                        path.display()
                    );
                }
                if provenance.human_evidence.is_some() {
                    bail!(
                        "{} corpus line {line} AI blind-panel annotation cannot include humanEvidence",
                        path.display()
                    );
                }
                if matches!(
                    case.annotation_status,
                    None | Some(CorpusAnnotationStatus::Unreviewed)
                ) {
                    bail!(
                        "{} corpus line {line} AI blind-panel annotation must be reviewed, adjudicated, or ambiguous",
                        path.display()
                    );
                }
            }
            CorpusAnnotationOrigin::HumanIndependent => {
                if provenance.reviewer_type.as_deref() != Some("human") {
                    bail!(
                        "{} corpus line {line} independent human annotation requires reviewerType `human`",
                        path.display()
                    );
                }
                if provenance.human_evidence.is_none() {
                    bail!(
                        "{} corpus line {line} independent human annotation requires humanEvidence",
                        path.display()
                    );
                }
                if matches!(
                    case.annotation_status,
                    None | Some(CorpusAnnotationStatus::Unreviewed)
                ) {
                    bail!(
                        "{} corpus line {line} independent human annotation must be reviewed, adjudicated, or ambiguous",
                        path.display()
                    );
                }
            }
            CorpusAnnotationOrigin::SourceRevision => {
                if provenance.reviewer_type.as_deref() == Some("ai") {
                    bail!(
                        "{} corpus line {line} source revision cannot be labeled as an AI review",
                        path.display()
                    );
                }
            }
        }
    } else if case.review_provenance.is_some() || case.annotation_status.is_some() {
        bail!(
            "{} corpus line {line} annotationStatus/reviewProvenance requires annotationOrigin",
            path.display()
        );
    }

    if !requires_external_metadata {
        return Ok(());
    }
    if case
        .genre
        .as_deref()
        .is_none_or(|genre| genre.trim().is_empty())
    {
        bail!(
            "{} corpus line {line} non-project origin requires a non-empty genre",
            path.display()
        );
    }
    if case
        .document_id
        .as_deref()
        .is_none_or(|document_id| document_id.trim().is_empty())
    {
        bail!(
            "{} corpus line {line} non-project origin requires a documentId",
            path.display()
        );
    }
    if case.split.is_none() {
        bail!(
            "{} corpus line {line} non-project origin requires a split",
            path.display()
        );
    }
    if case
        .author_id
        .as_deref()
        .is_some_and(|author_id| author_id.trim().is_empty())
    {
        bail!(
            "{} corpus line {line} authorId cannot be empty",
            path.display()
        );
    }
    if case
        .error_families
        .iter()
        .any(|family| family.trim().is_empty())
    {
        bail!(
            "{} corpus line {line} errorFamilies cannot contain empty values",
            path.display()
        );
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // Keeps streaming parsing, validation, matching, and metric aggregation together.
#[allow(clippy::cast_precision_loss)]
fn evaluate_corpus(
    path: &Path,
    base_config: &LintConfig,
    provenance: Option<CorpusProvenance>,
    packs: &[RulePack],
    quality_gate: Option<&CorpusQualityGate>,
) -> Result<bool> {
    let file = fs::File::open(path)
        .with_context(|| format!("{} corpus file could not be opened", path.display()))?;
    let reader = BufReader::new(file);
    let mut cases = 0_usize;
    let mut true_positives = 0_usize;
    let mut false_positives = 0_usize;
    let mut false_negatives = 0_usize;
    let mut normal_cases = 0_usize;
    let mut false_positive_cases = 0_usize;
    let mut case_failures = Vec::new();
    let mut rule_metric_counts = BTreeMap::<String, RuleMetricCounts>::new();
    let mut seen_case_ids = BTreeMap::<String, usize>::new();
    let mut metadata_cases = Vec::new();
    let mut correction_cases = 0_usize;
    let mut top1_correction_hits = 0_usize;
    let mut top5_correction_hits = 0_usize;
    let mut engines: [Option<Engine>; 3] = [None, None, None];
    let mut has_fixed_text_mismatch = false;

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "{} corpus line {} could not be read as UTF-8",
                path.display(),
                index + 1
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let case: CorpusCase = serde_json::from_str(&line).with_context(|| {
            format!(
                "{} corpus line {} is not valid JSON Lines corpus data",
                path.display(),
                index + 1
            )
        })?;
        if case.id.trim().is_empty() {
            bail!(
                "{} corpus line {} has an empty case id",
                path.display(),
                index + 1
            );
        }
        let normalized_id = case.id.trim().to_owned();
        if let Some(previous_line) = seen_case_ids.insert(normalized_id.clone(), index + 1) {
            bail!(
                "{} corpus line {} has duplicate case id `{}` (first seen on line {})",
                path.display(),
                index + 1,
                normalized_id,
                previous_line
            );
        }
        for (field, value) in [
            ("genre", case.genre.as_deref()),
            ("provenanceId", case.provenance_id.as_deref()),
            ("sourceId", case.source_id.as_deref()),
            ("sourceSha256", case.source_sha256.as_deref()),
            ("sourceUrl", case.source_url.as_deref()),
            ("license", case.license.as_deref()),
        ] {
            if value.is_some_and(|value| value.trim().is_empty()) {
                bail!(
                    "{} corpus line {} has an empty {field}",
                    path.display(),
                    index + 1
                );
            }
        }
        if case.source_sha256.is_some() && !is_sha256(case.source_sha256.as_ref()) {
            bail!(
                "{} corpus line {} sourceSha256 must be 64 lowercase hexadecimal characters",
                path.display(),
                index + 1
            );
        }
        validate_corpus_metadata(path, index + 1, &case)?;

        let mut config = base_config.clone();
        if let Some(profile) = case.profile {
            config.profile = profile;
        }
        if !case.expected_rule_ids.is_empty() && !case.expected_diagnostics.is_empty() {
            bail!(
                "{} corpus line {} cannot use both expectedRuleIds and expectedDiagnostics",
                path.display(),
                index + 1
            );
        }
        let mut expected_diagnostics = if case.expected_diagnostics.is_empty() {
            case.expected_rule_ids
                .iter()
                .map(|rule_id| CorpusExpectedDiagnostic {
                    rule_id: rule_id.clone(),
                    original: None,
                    range: None,
                    suggestions: None,
                })
                .collect()
        } else {
            case.expected_diagnostics.clone()
        };
        validate_corpus_case_type(path, index + 1, case.case_type, expected_diagnostics.len())?;
        resolve_expected_diagnostic_ranges(path, index + 1, &case.text, &mut expected_diagnostics)?;
        metadata_cases.push(CaseMetadata {
            id: case.id.clone(),
            origin: case.origin,
            text_origin: case.text_origin,
            annotation_origin: case.annotation_origin,
            annotation_status: case.annotation_status,
            split: case.split,
            holdout_id: case.holdout_id.clone(),
            genre: case.genre.clone(),
            document_id: case.document_id.clone(),
            author_id: case.author_id.clone(),
            error_families: case.error_families.clone(),
            normal: expected_diagnostics.is_empty(),
        });
        let engine_index = profile_engine_index(config.profile);
        if engines[engine_index].is_none() {
            engines[engine_index] = Some(build_engine(config, packs.to_vec())?);
        }
        let engine = engines[engine_index]
            .as_ref()
            .expect("engine is initialized for the selected profile");
        let outcome = engine.check_with_fixes(&case.text, case.source_kind, false);
        let actual_diagnostics = outcome.diagnostics;
        let fixed_text_mismatch = case.expected_fixed_text.as_ref().and_then(|expected| {
            let actual = outcome.fixed_text.clone();
            (actual != *expected).then(|| CorpusFixedTextMismatch {
                expected: expected.clone(),
                actual,
            })
        });
        has_fixed_text_mismatch |= fixed_text_mismatch.is_some();
        if expected_diagnostics.is_empty() {
            normal_cases += 1;
            if !actual_diagnostics.is_empty() {
                false_positive_cases += 1;
            }
        }
        let comparison = compare_diagnostics(&expected_diagnostics, &actual_diagnostics);
        let correction = correction_accuracy(&expected_diagnostics, &actual_diagnostics);
        correction_cases += correction.cases;
        top1_correction_hits += correction.top1_hits;
        top5_correction_hits += correction.top5_hits;
        true_positives += comparison.true_positives;
        false_positives += comparison.false_positive_rule_ids.len();
        false_negatives += comparison.false_negative_rule_ids.len();
        for rule_id in &comparison.true_positive_rule_ids {
            rule_metric_counts
                .entry(rule_id.clone())
                .or_default()
                .true_positives += 1;
        }
        for rule_id in &comparison.false_positive_rule_ids {
            rule_metric_counts
                .entry(rule_id.clone())
                .or_default()
                .false_positives += 1;
        }
        for rule_id in &comparison.false_negative_rule_ids {
            rule_metric_counts
                .entry(rule_id.clone())
                .or_default()
                .false_negatives += 1;
        }
        if !comparison.false_positive_rule_ids.is_empty()
            || !comparison.false_negative_rule_ids.is_empty()
            || fixed_text_mismatch.is_some()
        {
            case_failures.push(CorpusCaseFailure {
                id: case.id,
                false_positive_rule_ids: comparison.false_positive_rule_ids,
                false_negative_rule_ids: comparison.false_negative_rule_ids,
                fixed_text_mismatch,
            });
        }
        cases += 1;
    }

    if cases == 0 {
        bail!("{} corpus does not contain any cases", path.display());
    }

    let precision_denominator = true_positives + false_positives;
    let recall_denominator = true_positives + false_negatives;
    let clean_normal_cases = normal_cases - false_positive_cases;
    let dataset = aggregate_metadata(&metadata_cases);
    let rule_metrics: Vec<_> = rule_metric_counts
        .into_iter()
        .map(|(rule_id, counts)| {
            let predicted = counts.true_positives + counts.false_positives;
            let expected = counts.true_positives + counts.false_negatives;
            RuleCorpusMetric {
                rule_id,
                true_positives: counts.true_positives,
                false_positives: counts.false_positives,
                false_negatives: counts.false_negatives,
                precision: ratio(counts.true_positives, predicted),
                recall: ratio(counts.true_positives, expected),
                precision_wilson_lower_95: wilson_lower_bound(counts.true_positives, predicted),
            }
        })
        .collect();
    let mut report = CorpusReport {
        version: 1,
        provenance,
        cases,
        true_positives,
        false_positives,
        false_negatives,
        precision: ratio(true_positives, precision_denominator),
        recall: ratio(true_positives, recall_denominator),
        macro_precision: mean(rule_metrics.iter().filter_map(|metric| metric.precision)),
        macro_recall: mean(rule_metrics.iter().filter_map(|metric| metric.recall)),
        normal_cases,
        false_positive_cases,
        specificity: ratio(clean_normal_cases, normal_cases),
        correction_cases,
        top1_correction_accuracy: ratio(top1_correction_hits, correction_cases),
        top5_correction_accuracy: ratio(top5_correction_hits, correction_cases),
        dataset,
        rule_metrics,
        quality_gate: None,
        case_failures,
    };
    let has_failure = if let Some(gate) = quality_gate {
        let mut gate_report = evaluate_corpus_quality_gate(&report, gate);
        gate_report.failures.extend(
            evaluate_dataset_gate(&report.dataset, &gate.dataset)
                .into_iter()
                .map(|failure| CorpusQualityGateFailure {
                    metric: failure.metric,
                    actual: Some(failure.actual as f64),
                    minimum: failure.minimum as f64,
                    rule_id: None,
                    holdout_id: failure.holdout_id,
                }),
        );
        gate_report.passed = gate_report.failures.is_empty();
        let passed = gate_report.passed;
        report.quality_gate = Some(gate_report);
        !passed || has_fixed_text_mismatch
    } else {
        report.false_positives > 0 || report.false_negatives > 0 || has_fixed_text_mismatch
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(has_failure)
}

fn load_corpus_quality_gate(path: &Path) -> Result<CorpusQualityGate> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("{} corpus gate 파일을 읽을 수 없습니다", path.display()))?;
    let mut gate: CorpusQualityGate = serde_json::from_str(&contents).with_context(|| {
        format!(
            "{} corpus gate 형식이 올바른 JSON이 아닙니다",
            path.display()
        )
    })?;
    if gate.schema_version != 1 {
        bail!("{} corpus gate schemaVersion must be 1", path.display());
    }
    for (field, value) in [
        ("minMicroPrecision", gate.min_micro_precision),
        ("minMacroPrecision", gate.min_macro_precision),
        ("minRecall", gate.min_recall),
        (
            "minTop1CorrectionAccuracy",
            gate.min_top1_correction_accuracy.unwrap_or(0.0),
        ),
        (
            "minTop5CorrectionAccuracy",
            gate.min_top5_correction_accuracy.unwrap_or(0.0),
        ),
        (
            "minRulePrecisionWilsonLower95",
            gate.min_rule_precision_wilson_lower_95,
        ),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            bail!(
                "{} corpus gate {field} must be a number from 0 to 1",
                path.display()
            );
        }
    }
    if gate.min_expected_per_rule == 0 {
        bail!(
            "{} corpus gate minExpectedPerRule must be at least 1",
            path.display()
        );
    }
    if gate.required_rule_ids.is_empty()
        || gate.required_rule_ids.iter().any(|id| id.trim().is_empty())
    {
        bail!(
            "{} corpus gate requiredRuleIds must contain non-empty rule IDs",
            path.display()
        );
    }
    let required_rule_id_count = gate.required_rule_ids.len();
    gate.required_rule_ids.sort_unstable();
    gate.required_rule_ids.dedup();
    if gate.required_rule_ids.len() != required_rule_id_count {
        bail!(
            "{} corpus gate requiredRuleIds must not repeat a rule ID",
            path.display()
        );
    }
    if gate.dataset.min_holdout_cases.is_some() {
        if gate.dataset.required_holdout_ids.is_empty()
            || gate
                .dataset
                .required_holdout_ids
                .iter()
                .any(|id| !matches!(id.as_str(), "H1" | "H2"))
        {
            bail!(
                "{} corpus gate requiredHoldoutIds must contain H1 and/or H2",
                path.display()
            );
        }
        let holdout_id_count = gate.dataset.required_holdout_ids.len();
        let mut holdout_ids = gate.dataset.required_holdout_ids.clone();
        holdout_ids.sort_unstable();
        holdout_ids.dedup();
        if holdout_ids.len() != holdout_id_count {
            bail!(
                "{} corpus gate requiredHoldoutIds must not repeat a holdout ID",
                path.display()
            );
        }
        if holdout_ids != vec!["H1".to_owned(), "H2".to_owned()] {
            bail!(
                "{} corpus gate requiredHoldoutIds must include both H1 and H2",
                path.display()
            );
        }
    }
    Ok(gate)
}

#[allow(clippy::cast_precision_loss)]
fn evaluate_corpus_quality_gate(
    report: &CorpusReport,
    gate: &CorpusQualityGate,
) -> CorpusQualityGateReport {
    let mut failures = Vec::new();
    push_gate_failure(
        &mut failures,
        "microPrecision",
        report.precision,
        gate.min_micro_precision,
        None,
    );
    push_gate_failure(
        &mut failures,
        "macroPrecision",
        report.macro_precision,
        gate.min_macro_precision,
        None,
    );
    push_gate_failure(
        &mut failures,
        "recall",
        report.recall,
        gate.min_recall,
        None,
    );
    if let Some(minimum) = gate.min_top1_correction_accuracy {
        push_gate_failure(
            &mut failures,
            "top1CorrectionAccuracy",
            report.top1_correction_accuracy,
            minimum,
            None,
        );
    }
    if let Some(minimum) = gate.min_top5_correction_accuracy {
        push_gate_failure(
            &mut failures,
            "top5CorrectionAccuracy",
            report.top5_correction_accuracy,
            minimum,
            None,
        );
    }
    for required_rule_id in &gate.required_rule_ids {
        let metric = report
            .rule_metrics
            .iter()
            .find(|metric| metric.rule_id == *required_rule_id);
        let expected = metric.map_or(0, |metric| metric.true_positives + metric.false_negatives);
        let rule_id = Some(required_rule_id.clone());
        push_gate_failure(
            &mut failures,
            "expectedCases",
            Some(expected as f64),
            gate.min_expected_per_rule as f64,
            rule_id.clone(),
        );
        if expected >= gate.min_expected_per_rule {
            push_gate_failure(
                &mut failures,
                "precisionWilsonLower95",
                metric.and_then(|metric| metric.precision_wilson_lower_95),
                gate.min_rule_precision_wilson_lower_95,
                rule_id,
            );
        }
    }
    CorpusQualityGateReport {
        passed: failures.is_empty(),
        failures,
    }
}

fn push_gate_failure(
    failures: &mut Vec<CorpusQualityGateFailure>,
    metric: &'static str,
    actual: Option<f64>,
    minimum: f64,
    rule_id: Option<String>,
) {
    if actual.is_none_or(|value| value < minimum) {
        failures.push(CorpusQualityGateFailure {
            metric,
            actual,
            minimum,
            rule_id,
            holdout_id: None,
        });
    }
}

fn load_corpus_manifest(manifest_path: &Path) -> Result<(PathBuf, CorpusProvenance)> {
    let manifest_contents = fs::read_to_string(manifest_path).with_context(|| {
        format!(
            "{} corpus manifest could not be read as UTF-8",
            manifest_path.display()
        )
    })?;
    let manifest: CorpusManifest = serde_json::from_str(&manifest_contents).with_context(|| {
        format!(
            "{} corpus manifest is not valid JSON",
            manifest_path.display()
        )
    })?;
    if manifest.schema_version != 1 {
        bail!(
            "{} corpus manifest schemaVersion must be 1",
            manifest_path.display()
        );
    }
    for (field, value) in [
        ("name", manifest.name.as_str()),
        ("license", manifest.license.as_str()),
        ("sourceUrl", manifest.source_url.as_str()),
        ("sha256", manifest.sha256.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!(
                "{} corpus manifest has an empty {field}",
                manifest_path.display()
            );
        }
    }
    let expected_sha256 = manifest.sha256.to_ascii_lowercase();
    if expected_sha256.len() != 64 || !expected_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!(
            "{} corpus manifest sha256 must be 64 hexadecimal characters",
            manifest_path.display()
        );
    }
    let corpus_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.corpus_path);
    let corpus_bytes = fs::read(&corpus_path).with_context(|| {
        format!(
            "{} corpus artifact referenced by the manifest could not be read",
            corpus_path.display()
        )
    })?;
    let actual_sha256 = sha256_hex(&corpus_bytes);
    if actual_sha256 != expected_sha256 {
        bail!(
            "{} corpus SHA-256 does not match its manifest",
            corpus_path.display()
        );
    }
    Ok((
        corpus_path,
        CorpusProvenance {
            name: manifest.name,
            license: manifest.license,
            source_url: manifest.source_url,
            sha256: actual_sha256,
        },
    ))
}

struct DiagnosticComparison {
    true_positives: usize,
    true_positive_rule_ids: Vec<String>,
    false_positive_rule_ids: Vec<String>,
    false_negative_rule_ids: Vec<String>,
}

#[derive(Default)]
struct CorrectionAccuracyCounts {
    cases: usize,
    top1_hits: usize,
    top5_hits: usize,
}

fn correction_accuracy(
    expected: &[CorpusExpectedDiagnostic],
    actual: &[Diagnostic],
) -> CorrectionAccuracyCounts {
    let mut counts = CorrectionAccuracyCounts::default();
    for expectation in expected {
        let Some(suggestions) = expectation
            .suggestions
            .as_ref()
            .filter(|suggestions| !suggestions.is_empty())
        else {
            continue;
        };
        counts.cases += 1;
        let Some(diagnostic) = actual.iter().find(|diagnostic| {
            expectation.rule_id == diagnostic.rule_id
                && expectation
                    .range
                    .as_ref()
                    .is_none_or(|range| range == &diagnostic.range)
        }) else {
            continue;
        };
        if diagnostic
            .suggestions
            .first()
            .is_some_and(|suggestion| suggestions.contains(suggestion))
        {
            counts.top1_hits += 1;
        }
        if diagnostic
            .suggestions
            .iter()
            .take(5)
            .any(|suggestion| suggestions.contains(suggestion))
        {
            counts.top5_hits += 1;
        }
    }
    counts
}

fn compare_diagnostics(
    expected: &[CorpusExpectedDiagnostic],
    actual: &[Diagnostic],
) -> DiagnosticComparison {
    let mut expected_indices: Vec<_> = (0..expected.len()).collect();
    expected_indices.sort_by(|left, right| {
        diagnostic_constraint_count(&expected[*right])
            .cmp(&diagnostic_constraint_count(&expected[*left]))
    });
    let mut expected_matches = vec![false; expected.len()];
    let mut actual_matches = vec![false; actual.len()];
    let mut true_positives = 0_usize;
    let mut true_positive_rule_ids = Vec::new();

    for expected_index in expected_indices {
        let expectation = &expected[expected_index];
        if let Some(actual_index) = actual.iter().enumerate().find_map(|(index, diagnostic)| {
            (!actual_matches[index] && diagnostic_matches(expectation, diagnostic)).then_some(index)
        }) {
            expected_matches[expected_index] = true;
            actual_matches[actual_index] = true;
            true_positives += 1;
            true_positive_rule_ids.push(expectation.rule_id.clone());
        }
    }

    DiagnosticComparison {
        true_positives,
        true_positive_rule_ids,
        false_positive_rule_ids: actual
            .iter()
            .zip(actual_matches)
            .filter(|(_, matched)| !*matched)
            .map(|(diagnostic, _)| diagnostic.rule_id.clone())
            .collect(),
        false_negative_rule_ids: expected
            .iter()
            .zip(expected_matches)
            .filter(|(_, matched)| !*matched)
            .map(|(diagnostic, _)| diagnostic.rule_id.clone())
            .collect(),
    }
}

fn diagnostic_constraint_count(expectation: &CorpusExpectedDiagnostic) -> u8 {
    u8::from(expectation.range.is_some()) + u8::from(expectation.suggestions.is_some())
}

fn diagnostic_matches(expectation: &CorpusExpectedDiagnostic, diagnostic: &Diagnostic) -> bool {
    expectation.rule_id == diagnostic.rule_id
        && expectation
            .range
            .as_ref()
            .is_none_or(|range| range == &diagnostic.range)
        && expectation
            .suggestions
            .as_ref()
            .is_none_or(|suggestions| suggestions == &diagnostic.suggestions)
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[allow(clippy::cast_precision_loss)] // JSON metrics are ratios; corpus counts are practical-sized line totals.
fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

#[allow(clippy::cast_precision_loss)]
fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let (sum, count) = values.fold((0.0, 0_usize), |(sum, count), value| {
        (sum + value, count + 1)
    });
    (count > 0).then(|| sum / count as f64)
}

#[allow(clippy::cast_precision_loss)]
fn wilson_lower_bound(successes: usize, trials: usize) -> Option<f64> {
    const Z_95: f64 = 1.959_963_984_540_054;
    if trials == 0 {
        return None;
    }
    let sample_size = trials as f64;
    let proportion = successes as f64 / sample_size;
    let z_squared = Z_95 * Z_95;
    let numerator = proportion + z_squared / (2.0 * sample_size)
        - Z_95
            * ((proportion * (1.0 - proportion) + z_squared / (4.0 * sample_size)) / sample_size)
                .sqrt();
    Some(numerator / (1.0 + z_squared / sample_size))
}

fn print_rule_explanation(rule_id: &str, format: OutputFormat) -> Result<()> {
    let metadata =
        rule_metadata(rule_id).with_context(|| format!("{rule_id} 규칙을 찾을 수 없습니다"))?;
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&metadata)?),
        OutputFormat::Human => {
            let profiles = metadata
                .profiles
                .iter()
                .copied()
                .map(profile_name)
                .collect::<Vec<_>>()
                .join(", ");
            println!("{}", metadata.id);
            println!("  category: {}", metadata.category);
            println!("  confidence: {}", confidence_name(metadata.confidence));
            println!("  fix safety: {}", fix_safety_name(metadata.fix_safety));
            println!("  profiles: {profiles}");
            println!("  docs: {}", metadata.documentation_url);
        }
        OutputFormat::Sarif => bail!("--explain에는 --format human 또는 json을 사용하세요"),
    }
    Ok(())
}

fn profile_name(profile: Profile) -> &'static str {
    match profile {
        Profile::Default => "default",
        Profile::Strict => "strict",
        Profile::Editorial => "editorial",
    }
}

fn confidence_name(confidence: geullint_core::Confidence) -> &'static str {
    match confidence {
        geullint_core::Confidence::High => "high",
        geullint_core::Confidence::Medium => "medium",
        geullint_core::Confidence::Low => "low",
    }
}

fn fix_safety_name(fix_safety: geullint_core::FixSafety) -> &'static str {
    match fix_safety {
        geullint_core::FixSafety::Safe => "safe",
        geullint_core::FixSafety::Review => "review",
        geullint_core::FixSafety::None => "none",
    }
}

fn load_config(arguments: &Arguments) -> Result<LintConfig> {
    let config_path = if let Some(path) = &arguments.config {
        Some(path.clone())
    } else {
        let default_path = PathBuf::from(".geullint.json");
        default_path.is_file().then_some(default_path)
    };
    let mut config = match config_path {
        Some(path) => {
            let contents = fs::read_to_string(&path)
                .with_context(|| format!("{} 설정 파일을 읽을 수 없습니다", path.display()))?;
            serde_json::from_str(&contents)
                .with_context(|| format!("{} 설정 파일이 올바른 JSON이 아닙니다", path.display()))?
        }
        None => LintConfig::default(),
    };
    config
        .disabled_rules
        .extend(arguments.disabled_rules.iter().cloned());
    if let Some(profile) = arguments.profile {
        config.profile = profile.into();
    }
    for overlay_path in &arguments.dictionary_overlays {
        let contents = fs::read_to_string(overlay_path).with_context(|| {
            format!(
                "{} 사전 overlay 파일을 읽을 수 없습니다",
                overlay_path.display()
            )
        })?;
        let overlay = DictionaryOverlay::parse(&contents).with_context(|| {
            format!(
                "{} 사전 overlay 파일 형식이 올바르지 않습니다",
                overlay_path.display()
            )
        })?;
        config
            .dictionary_overlay
            .extend(overlay.surfaces().map(str::to_owned));
    }
    config.disabled_rules.sort_unstable();
    config.disabled_rules.dedup();
    config.dictionary_overlay.sort_unstable();
    config.dictionary_overlay.dedup();
    Ok(config)
}

fn load_rule_packs(arguments: &Arguments) -> Result<Vec<RulePack>> {
    let mut packs = Vec::new();
    for pack_path in &arguments.rule_packs {
        let source = fs::read_to_string(pack_path).with_context(|| {
            format!("{} 규칙 묶음 파일을 읽을 수 없습니다", pack_path.display())
        })?;
        let pack = RulePack::parse(&source).with_context(|| {
            format!("{} 규칙 묶음 형식이 올바르지 않습니다", pack_path.display())
        })?;
        packs.push(pack);
    }
    Ok(packs)
}

fn build_engine(config: LintConfig, packs: Vec<RulePack>) -> Result<Engine> {
    Engine::with_rule_packs(config, packs).context("규칙 묶음 ID가 기존 규칙과 충돌합니다")
}

fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            let mut builder = WalkBuilder::new(path);
            builder
                .add_custom_ignore_filename(".geullintignore")
                .require_git(false)
                .follow_links(false)
                .hidden(false)
                .filter_entry(|entry| !is_ignored_directory(entry.path()));
            for entry in builder.build() {
                let entry =
                    entry.with_context(|| format!("{} 경로를 읽을 수 없습니다", path.display()))?;
                if entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_file())
                {
                    let entry_path = entry.into_path();
                    if supported_source_kind(&entry_path).is_some()
                        && contains_valid_utf8(&entry_path)?
                    {
                        files.push(entry_path);
                    }
                }
            }
        } else {
            bail!("{} 경로를 찾을 수 없습니다", path.display());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn is_ignored_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git"
                    | ".next"
                    | ".turbo"
                    | ".worktrees"
                    | "coverage"
                    | "dist"
                    | "node_modules"
                    | "target"
            )
        })
}

fn source_kind_for_path(path: &Path) -> SourceKind {
    supported_source_kind(path).unwrap_or(SourceKind::PlainText)
}

fn supported_source_kind(path: &Path) -> Option<SourceKind> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())?
        .to_ascii_lowercase();
    match extension.as_str() {
        "md" | "markdown" => Some(SourceKind::Markdown),
        "js" | "jsx" | "mjs" | "cjs" => Some(SourceKind::JavaScript),
        "ts" | "tsx" | "mts" | "cts" => Some(SourceKind::TypeScript),
        "py" => Some(SourceKind::Python),
        "rs" => Some(SourceKind::Rust),
        "txt" | "text" => Some(SourceKind::PlainText),
        _ => None,
    }
}

fn contains_valid_utf8(path: &Path) -> Result<bool> {
    let bytes =
        fs::read(path).with_context(|| format!("{} 파일을 읽을 수 없습니다", path.display()))?;
    Ok(std::str::from_utf8(&bytes).is_ok())
}

fn line_and_column(text: &str, byte_offset: usize) -> (usize, usize) {
    let prefix = &text[..byte_offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, line)| line)
        .chars()
        .count()
        + 1;
    (line, column)
}

fn reaches_threshold(severity: Severity, threshold: FailOn) -> bool {
    matches!(
        (severity, threshold),
        (Severity::Error, _)
            | (Severity::Warning, FailOn::Warning | FailOn::Info)
            | (Severity::Info, FailOn::Info)
    )
}

fn print_report(
    reported: &[ReportedDiagnostic],
    format: OutputFormat,
    no_color: bool,
) -> Result<()> {
    // Human, JSON, and SARIF output intentionally contain no ANSI escapes. Reading the flag here
    // keeps `--no-color` an explicit, tested contract without introducing a styling dependency.
    let _ = no_color;
    match format {
        OutputFormat::Human => {
            for finding in reported {
                let diagnostic = &finding.diagnostic;
                let suggestion = diagnostic
                    .suggestions
                    .first()
                    .map_or(String::new(), |value| format!(" → {value}"));
                println!(
                    "{}:{}:{}: {} [{}] {}{}",
                    finding.path,
                    finding.line,
                    finding.column,
                    severity_name(diagnostic.severity),
                    diagnostic.rule_id,
                    diagnostic.message,
                    suggestion
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&JsonReport {
                version: 1,
                diagnostics: reported.to_vec(),
            })?
        ),
        OutputFormat::Sarif => print_sarif(reported)?,
    }
    Ok(())
}

fn print_sarif(reported: &[ReportedDiagnostic]) -> Result<()> {
    let report = SarifLog {
        version: "2.1.0",
        schema: "https://json.schemastore.org/sarif-2.1.0.json",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "GeulLint",
                    information_uri: "https://github.com/binibinibin123/geullint",
                },
            },
            column_kind: "unicodeCodePoints",
            results: reported
                .iter()
                .map(|finding| SarifResult {
                    rule_id: finding.diagnostic.rule_id.clone(),
                    level: sarif_level(finding.diagnostic.severity),
                    message: SarifMessage {
                        text: finding.diagnostic.message.clone(),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: sarif_artifact_uri(&finding.path),
                            },
                            region: SarifRegion {
                                start_line: finding.line,
                                start_column: finding.column,
                            },
                        },
                    }],
                })
                .collect(),
        }],
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn sarif_artifact_uri(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let bytes = normalized.as_bytes();
    let is_windows_absolute =
        bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/';
    let encoded = percent_encode_uri_path(&normalized, is_windows_absolute);

    if is_windows_absolute {
        format!("file:///{encoded}")
    } else if normalized.starts_with("//") {
        format!("file:{encoded}")
    } else if normalized.starts_with('/') {
        format!("file://{encoded}")
    } else {
        encoded
    }
}

fn percent_encode_uri_path(path: &str, preserve_colon: bool) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/')
            || (preserve_colon && byte == b':')
        {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to a string cannot fail");
        }
    }
    encoded
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}
