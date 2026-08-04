#![forbid(unsafe_code)]

use geullint_core::{
    DictionaryOverlay, Engine, LintConfig, RuleMetadata, RulePack, Severity, SourceKind,
    rule_catalog,
};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    ls_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, Diagnostic, DiagnosticSeverity,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
        NumberOrString, Position, Range, ServerCapabilities, TextDocumentContentChangeEvent,
        TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Uri, WorkspaceEdit,
    },
};

/// Serves the `GeulLint` language server through standard input/output.
pub async fn run_stdio() {
    let (service, socket) = LspService::build(Backend::new)
        .custom_method("geullint/rules", Backend::rules)
        .finish();
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

#[derive(Clone, Debug)]
struct DocumentState {
    text: String,
    source_kind: SourceKind,
    version: i32,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    engine: RwLock<Engine>,
    documents: RwLock<HashMap<Uri, DocumentState>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            engine: RwLock::new(Engine::new(LintConfig::default())),
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn publish(&self, uri: Uri, state: &DocumentState) {
        let engine = self.engine.read().await.clone();
        let diagnostics = engine
            .check(&state.text, state.source_kind)
            .into_iter()
            .map(|diagnostic| {
                let suggestion = diagnostic
                    .suggestions
                    .first()
                    .map_or(String::new(), |value| format!(" 제안: {value}"));
                Diagnostic {
                    range: Range {
                        start: position_for_byte_offset(&state.text, diagnostic.range.start),
                        end: position_for_byte_offset(&state.text, diagnostic.range.end),
                    },
                    severity: Some(severity_for_lsp(diagnostic.severity)),
                    code: Some(NumberOrString::String(diagnostic.rule_id)),
                    source: Some("geullint".to_owned()),
                    message: format!("{}{}", diagnostic.message, suggestion),
                    data: diagnostic.suggestions.first().map(|replacement| {
                        serde_json::json!({
                            "replacement": replacement,
                            "safeFix": diagnostic.safe_fix,
                        })
                    }),
                    ..Diagnostic::default()
                }
            })
            .collect();
        self.client
            .publish_diagnostics(uri, diagnostics, Some(state.version))
            .await;
    }

    async fn republish_documents(&self) {
        let documents = self.documents.read().await.clone();
        for (uri, state) in documents {
            self.publish(uri, &state).await;
        }
    }

    async fn update_engine(&self, value: serde_json::Value) {
        match engine_from_lsp_value(value) {
            Ok(engine) => {
                *self.engine.write().await = engine;
                self.republish_documents().await;
            }
            Err(error) => {
                self.client
                    .log_message(
                        tower_lsp_server::ls_types::MessageType::WARNING,
                        format!("GeulLint 설정을 적용하지 않았습니다: {error}"),
                    )
                    .await;
            }
        }
    }

    #[allow(clippy::unused_async)] // tower-lsp custom methods use an async callback signature.
    async fn rules(&self) -> Result<Vec<RuleMetadata>> {
        Ok(rule_catalog_response())
    }
}

fn rule_catalog_response() -> Vec<RuleMetadata> {
    rule_catalog()
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(value) = params.initialization_options {
            self.update_engine(value).await;
        }
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                tower_lsp_server::ls_types::MessageType::INFO,
                "GeulLint: 완전 오프라인 검사기를 시작했습니다.",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        let uri = document.uri.clone();
        let state = DocumentState {
            source_kind: source_kind_for_uri(&uri),
            text: document.text,
            version: document.version,
        };
        self.documents.write().await.insert(uri.clone(), state);
        let state = self.documents.read().await.get(&uri).cloned();
        if let Some(state) = state {
            self.publish(uri, &state).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if params.content_changes.is_empty() {
            return;
        }
        let previous = self.documents.read().await.get(&uri).cloned();
        let Some(previous) = previous else {
            return;
        };
        let text = match apply_content_changes(&previous.text, &params.content_changes) {
            Ok(text) => text,
            Err(error) => {
                self.client
                    .log_message(
                        tower_lsp_server::ls_types::MessageType::WARNING,
                        format!("GeulLint 증분 변경을 적용하지 못했습니다: {error}"),
                    )
                    .await;
                return;
            }
        };
        let state = DocumentState {
            text,
            source_kind: previous.source_kind,
            version: params.text_document.version,
        };
        self.documents.write().await.insert(uri.clone(), state);
        let state = self.documents.read().await.get(&uri).cloned();
        if let Some(state) = state {
            self.publish(uri, &state).await;
        }
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.update_engine(params.settings).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents
            .write()
            .await
            .remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let actions: Vec<CodeActionOrCommand> = params
            .context
            .diagnostics
            .iter()
            .filter_map(|diagnostic| {
                quick_fix_for_diagnostic(&params.text_document.uri, diagnostic)
            })
            .map(CodeActionOrCommand::CodeAction)
            .collect();
        Ok(Some(actions))
    }
}

fn quick_fix_for_diagnostic(uri: &Uri, diagnostic: &Diagnostic) -> Option<CodeAction> {
    let data = diagnostic.data.as_ref()?;
    if !data.get("safeFix")?.as_bool()? {
        return None;
    }
    let replacement = data.get("replacement")?.as_str()?.to_owned();
    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit::new(diagnostic.range, replacement.clone())],
    );
    Some(CodeAction {
        title: format!("‘{replacement}’로 고치기"),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..WorkspaceEdit::default()
        }),
        is_preferred: Some(true),
        ..CodeAction::default()
    })
}

fn source_kind_for_uri(uri: &Uri) -> SourceKind {
    match uri
        .to_file_path()
        .as_deref()
        .and_then(|path| path.extension())
        .and_then(|extension| extension.to_str())
    {
        Some("md" | "mdx") => SourceKind::Markdown,
        Some("js" | "jsx" | "mjs" | "cjs") => SourceKind::JavaScript,
        Some("ts" | "tsx" | "mts" | "cts") => SourceKind::TypeScript,
        Some("py") => SourceKind::Python,
        Some("rs") => SourceKind::Rust,
        _ => SourceKind::PlainText,
    }
}

fn severity_for_lsp(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
    }
}

fn position_for_byte_offset(text: &str, byte_offset: usize) -> Position {
    let prefix = &text[..byte_offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_prefix = prefix.rsplit_once('\n').map_or(prefix, |(_, line)| line);
    Position::new(
        u32::try_from(line).expect("line count fits LSP position"),
        u32::try_from(line_prefix.encode_utf16().count()).expect("column fits LSP position"),
    )
}

fn apply_content_changes(
    original: &str,
    changes: &[TextDocumentContentChangeEvent],
) -> std::result::Result<String, String> {
    let mut text = original.to_owned();
    for change in changes {
        let Some(range) = change.range else {
            text = change.text.clone();
            continue;
        };
        let start = byte_offset_for_position(&text, range.start)
            .ok_or_else(|| "증분 변경의 시작 위치가 텍스트 범위를 벗어났습니다".to_owned())?;
        let end = byte_offset_for_position(&text, range.end)
            .ok_or_else(|| "증분 변경의 끝 위치가 텍스트 범위를 벗어났습니다".to_owned())?;
        if start > end {
            return Err("증분 변경의 시작 위치가 끝 위치보다 큽니다".to_owned());
        }
        text.replace_range(start..end, &change.text);
    }
    Ok(text)
}

fn byte_offset_for_position(text: &str, position: Position) -> Option<usize> {
    let mut line = 0_u32;
    let mut line_start = 0_usize;
    for (offset, character) in text.char_indices() {
        if line == position.line {
            let column = text[line_start..offset].encode_utf16().count() as u32;
            if column == position.character {
                return Some(offset);
            }
            if column > position.character {
                return None;
            }
        }
        if character == '\n' {
            if line == position.line {
                return (position.character == 0).then_some(offset);
            }
            line += 1;
            line_start = offset + character.len_utf8();
        }
    }
    if line == position.line {
        let column = text[line_start..].encode_utf16().count() as u32;
        if column == position.character {
            return Some(text.len());
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)] // Configuration parsers remain below the focused LSP tests.
mod tests {
    use super::{
        apply_content_changes, engine_from_lsp_value, lint_config_from_lsp_value,
        position_for_byte_offset, quick_fix_for_diagnostic, rule_catalog_response,
    };
    use geullint_core::{Profile, SourceKind};
    use serde_json::json;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tower_lsp_server::ls_types::{
        CodeActionKind, Diagnostic, Position, Range, TextDocumentContentChangeEvent, Uri,
    };

    #[test]
    fn converts_utf8_offsets_to_utf16_lsp_positions() {
        let source = "첫😀줄\n몇일";

        assert_eq!(position_for_byte_offset(source, 7), Position::new(0, 3));
        assert_eq!(position_for_byte_offset(source, 14), Position::new(1, 1));
    }

    #[test]
    fn applies_incremental_utf16_changes_in_order() {
        let changes = vec![
            TextDocumentContentChangeEvent {
                range: Some(Range::new(Position::new(0, 1), Position::new(0, 3))),
                range_length: None,
                text: "X".to_owned(),
            },
            TextDocumentContentChangeEvent {
                range: Some(Range::new(Position::new(1, 0), Position::new(1, 1))),
                range_length: None,
                text: "Y".to_owned(),
            },
        ];
        assert_eq!(
            apply_content_changes("a😀\nbc", &changes).unwrap(),
            "aX\nYc"
        );
    }

    #[test]
    fn rejects_incremental_changes_at_non_boundary_positions() {
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 2), Position::new(0, 3))),
            range_length: None,
            text: "X".to_owned(),
        }];
        assert!(apply_content_changes("a😀", &changes).is_err());
    }

    #[test]
    fn creates_a_quick_fix_only_for_safe_replacements() {
        let uri: Uri = "file:///tmp/memo.txt".parse().expect("valid URI");
        let diagnostic = Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 2)),
            data: Some(json!({"replacement": "며칠", "safeFix": true})),
            ..Diagnostic::default()
        };

        let action = quick_fix_for_diagnostic(&uri, &diagnostic).expect("quick fix");

        assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
        assert_eq!(
            action.edit.expect("workspace edit").changes.unwrap()[&uri][0].new_text,
            "며칠"
        );
    }

    #[test]
    fn returns_the_ordered_curated_rule_catalogue_for_custom_requests() {
        let rules = rule_catalog_response();

        let declared_count: usize = include_str!("../../../rules/catalog-count.txt")
            .trim()
            .parse()
            .expect("catalog-count.txt must contain an integer");
        assert_eq!(rules.len(), declared_count);
        assert!(
            rules
                .windows(2)
                .all(|pair| pair[0].id.as_str() < pair[1].id.as_str())
        );
        assert_eq!(
            rules
                .iter()
                .find(|rule| rule.id == "spelling.lexical.myeochil")
                .map(|rule| rule.title.as_str()),
            Some("며칠 표기")
        );
    }

    #[test]
    fn accepts_profile_and_user_dictionary_from_editor_configuration() {
        let config = lint_config_from_lsp_value(json!({
            "geullint": {
                "profile": "editorial",
                "userDictionary": ["GeulLint"],
                "dictionaryOverlay": ["프로젝트오표기"]
            }
        }))
        .expect("valid editor configuration");

        assert_eq!(config.profile, Profile::Editorial);
        assert_eq!(config.user_dictionary, ["GeulLint"]);
        assert_eq!(config.dictionary_overlay, ["프로젝트오표기"]);
    }

    #[test]
    fn loads_local_dictionary_overlay_paths_from_editor_configuration() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "geullint-lsp-overlay-{}-{unique}.overlay",
            std::process::id()
        ));
        fs::write(&path, "geullint-overlay-v1\n몇일\tNNP\n").expect("write overlay");

        let engine = engine_from_lsp_value(json!({
            "geullint": {
                "dictionaryOverlayPaths": [path]
            }
        }))
        .expect("load editor dictionary overlay");
        fs::remove_file(&path).expect("remove overlay");

        assert!(engine.check("몇일", SourceKind::PlainText).is_empty());

        let invalid_path = std::env::temp_dir().join(format!(
            "geullint-lsp-invalid-overlay-{}-{unique}.overlay",
            std::process::id()
        ));
        fs::write(&invalid_path, "not-an-overlay\n몇일\tNNP\n").expect("write invalid overlay");
        let error = engine_from_lsp_value(json!({
            "geullint": {
                "dictionaryOverlayPaths": [invalid_path]
            }
        }))
        .expect_err("reject invalid editor dictionary overlay");
        fs::remove_file(invalid_path).expect("remove invalid overlay");

        assert!(error.contains("사전 overlay"));
    }

    #[test]
    fn loads_local_rule_packs_from_editor_configuration() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "geullint-lsp-rule-pack-{}-{unique}.yaml",
            std::process::id()
        ));
        fs::write(
            &path,
            "version: 1\nlanguage: ko\nrules:\n  - id: spelling.project.product-name\n    severity: warning\n    profile: default\n    message: 프로젝트 표기를 확인하세요.\n    safeFix: true\n    replacements:\n      - from: 글린트\n        to: GeulLint\n",
        )
        .expect("write rule pack");

        let engine = engine_from_lsp_value(json!({
            "geullint": {
                "rulePacks": [path]
            }
        }))
        .expect("load editor rule pack");
        let diagnostics = engine.check("글린트", SourceKind::PlainText);

        fs::remove_file(path).expect("remove rule pack");
        assert_eq!(diagnostics[0].rule_id, "spelling.project.product-name");
    }
}

fn lint_config_from_lsp_value(value: serde_json::Value) -> Option<LintConfig> {
    let settings = value.get("geullint").cloned().unwrap_or(value);
    serde_json::from_value(settings).ok()
}

fn engine_from_lsp_value(value: serde_json::Value) -> std::result::Result<Engine, String> {
    let settings = value.get("geullint").cloned().unwrap_or(value);
    let mut config = lint_config_from_lsp_value(settings.clone())
        .ok_or_else(|| "기본 설정 형식이 올바르지 않습니다".to_owned())?;
    let overlay_paths = settings
        .get("dictionaryOverlayPaths")
        .cloned()
        .map(serde_json::from_value::<Vec<String>>)
        .transpose()
        .map_err(|error| format!("dictionaryOverlayPaths는 파일 경로 목록이어야 합니다: {error}"))?
        .unwrap_or_default();
    for path in overlay_paths {
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("사전 overlay {path}을 읽을 수 없습니다: {error}"))?;
        let overlay = DictionaryOverlay::parse(&source)
            .map_err(|error| format!("사전 overlay {path} 형식이 올바르지 않습니다: {error}"))?;
        config
            .dictionary_overlay
            .extend(overlay.surfaces().map(str::to_owned));
    }
    config.dictionary_overlay.sort_unstable();
    config.dictionary_overlay.dedup();
    let paths = settings
        .get("rulePacks")
        .cloned()
        .map(serde_json::from_value::<Vec<String>>)
        .transpose()
        .map_err(|error| format!("rulePacks는 파일 경로 목록이어야 합니다: {error}"))?
        .unwrap_or_default();
    let packs = paths
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path)
                .map_err(|error| format!("규칙 팩 ‘{path}’을 읽을 수 없습니다: {error}"))?;
            RulePack::parse(&source)
                .map_err(|error| format!("규칙 팩 ‘{path}’이 올바르지 않습니다: {error}"))
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Engine::with_rule_packs(config, packs)
        .map_err(|error| format!("규칙 팩 구성이 충돌합니다: {error}"))
}
