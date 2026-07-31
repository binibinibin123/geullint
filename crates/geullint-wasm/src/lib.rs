#![forbid(unsafe_code)]

//! Browser bindings for the fully local `GeulLint` core.

use geullint_core::{
    Diagnostic, LintConfig, RuleMetadata, SourceKind, apply_safe_fixes, lint_text, rule_catalog,
};
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
    let diagnostics = lint_text(&request.text, request.source_kind, &request.config);
    let fixed_text = apply_safe_fixes(&request.text, &diagnostics);
    WasmLintResponse {
        version: 1,
        diagnostics,
        fixed_text,
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
