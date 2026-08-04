#![forbid(unsafe_code)]

//! Browser bindings for the fully local `GeulLint` core.

use geullint_core::{Diagnostic, Engine, LintConfig, RuleMetadata, SourceKind, rule_catalog};
#[cfg(feature = "standard")]
use geullint_core::{DiagnosticV2, StandardPipeline};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// The JSON request accepted by the browser-worker API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmLintRequest {
    pub text: String,
    pub source_kind: SourceKind,
    #[serde(default)]
    pub config: LintConfig,
    #[serde(default = "review_fixes_enabled_by_default")]
    pub include_review_fixes: bool,
}

const fn review_fixes_enabled_by_default() -> bool {
    true
}

impl WasmLintRequest {
    /// Parses a request supplied by JavaScript without performing any I/O.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not valid JSON for the documented API.
    pub fn from_json(source: &str) -> Result<Self, WasmRequestError> {
        serde_json::from_str(source).map_err(WasmRequestError::InvalidJson)
    }
}

/// The versioned, JSON-serializable response returned to browser callers.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmLintResponse {
    pub version: u8,
    pub diagnostics: Vec<Diagnostic>,
    pub fixed_text: String,
    pub review_fixed_text: String,
}

/// Response returned by the opt-in `standard` pipeline. Candidate suggestions remain Review
/// only until an independent release holdout calibrates their safe-fix precision.
#[cfg(feature = "standard")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardWasmLintResponse {
    pub version: u8,
    pub diagnostics: Vec<DiagnosticV2>,
    pub fixed_text: String,
    pub review_fixed_text: String,
}

/// The versioned catalogue shared by the native and browser builds.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmRuleCatalog {
    pub version: u8,
    pub rule_count: usize,
    pub rules: Vec<RuleMetadata>,
}

/// Parses and lints a browser request entirely in the current process.
#[must_use]
pub fn evaluate(request: &WasmLintRequest) -> WasmLintResponse {
    let engine = Engine::new(request.config.clone());
    let outcome = engine.check_with_fixes(
        &request.text,
        request.source_kind,
        request.include_review_fixes,
    );
    WasmLintResponse {
        version: 1,
        diagnostics: outcome.diagnostics,
        fixed_text: outcome.fixed_text,
        review_fixed_text: outcome.review_fixed_text,
    }
}

/// Evaluates the versioned standard lexicon and bounded candidate pipeline in WASM.
///
/// This entry point is separate from [`evaluate`] so compact native/browser parity remains a
/// byte-for-byte compatibility contract while standard candidates are still being calibrated.
///
/// # Panics
///
/// Panics only when the checked-in standard lexicon or ranker manifest is malformed. Release CI
/// validates both assets before packaging them.
#[cfg(feature = "standard")]
#[must_use]
pub fn evaluate_standard(request: &WasmLintRequest) -> StandardWasmLintResponse {
    let pipeline = StandardPipeline::bundled(request.config.clone())
        .expect("checked-in standard assets must be valid");
    let outcome = pipeline.check_with_fixes(
        &request.text,
        request.source_kind,
        request.include_review_fixes,
    );
    StandardWasmLintResponse {
        version: 1,
        diagnostics: outcome.diagnostics,
        fixed_text: outcome.fixed_text,
        review_fixed_text: outcome.review_fixed_text,
    }
}

/// Lints one JSON request and returns one JSON response for a Web Worker.
///
/// This binding does not open sockets, use browser storage, or transmit the text.
///
/// # Errors
///
/// Returns a JavaScript error when the request is invalid or serialization fails.
#[wasm_bindgen]
pub fn lint_json(request_json: &str) -> Result<String, JsValue> {
    let request = WasmLintRequest::from_json(request_json).map_err(js_error)?;
    serde_json::to_string(&evaluate(&request)).map_err(js_error)
}

/// Lints one JSON request with the opt-in standard pipeline.
///
/// # Errors
///
/// Returns a JavaScript error when the request is invalid JSON or the response cannot be
/// serialized.
#[cfg(feature = "standard")]
#[wasm_bindgen]
pub fn lint_standard_json(request_json: &str) -> Result<String, JsValue> {
    let request = WasmLintRequest::from_json(request_json).map_err(js_error)?;
    serde_json::to_string(&evaluate_standard(&request)).map_err(js_error)
}

/// Returns all bundled rule metadata for the offline playground.
///
/// # Errors
///
/// Returns a JavaScript error only when the in-memory catalogue cannot be serialized.
#[wasm_bindgen]
pub fn rule_catalog_json() -> Result<String, JsValue> {
    let rules = rule_catalog();
    serde_json::to_string(&WasmRuleCatalog {
        version: 1,
        rule_count: rules.len(),
        rules,
    })
    .map_err(js_error)
}

/// Reasons a browser request cannot be decoded.
#[derive(Debug, thiserror::Error)]
pub enum WasmRequestError {
    #[error("invalid GeulLint browser request")]
    InvalidJson(#[source] serde_json::Error),
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
