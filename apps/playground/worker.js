import init, {
  lint_context_json,
  lint_json,
  lint_standard_json,
  rule_catalog_json,
} from "./pkg/geullint_wasm.js";

const lintByEngine = Object.freeze({
  compact: lint_json,
  standard: lint_standard_json,
  context: lint_context_json,
});

function normalizeResponse(response) {
  return {
    ...response,
    diagnostics: (response.diagnostics || []).map((diagnostic) => ({
      ...diagnostic,
      safeFix: typeof diagnostic.safeFix === "boolean"
        ? diagnostic.safeFix
        : diagnostic.safety === "safe",
      suggestions: (diagnostic.suggestions || []).map((suggestion) => (
        typeof suggestion === "string" ? suggestion : suggestion.text
      )),
    })),
  };
}

const engine = init();
engine
  .then(() => postMessage({
    type: "ready",
    catalog: JSON.parse(rule_catalog_json()),
  }))
  .catch((error) => postMessage({ type: "error", message: String(error) }));

addEventListener("message", async ({ data }) => {
  try {
    await engine;
    const lint = lintByEngine[data.engine || "standard"];
    if (typeof lint !== "function") throw new Error(`unknown engine: ${data.engine}`);
    const response = normalizeResponse(JSON.parse(lint(JSON.stringify({
      text: data.text,
      sourceKind: data.sourceKind,
      config: data.config,
      includeReviewFixes: data.includeReviewFixes,
    }))));
    postMessage({ id: data.id, engine: data.engine || "standard", response });
  } catch (error) {
    postMessage({ type: "error", message: String(error) });
  }
});
