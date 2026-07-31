import init, { lint_json, rule_catalog_json } from "./pkg/geullint_wasm.js";

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
    const response = JSON.parse(lint_json(JSON.stringify({
      text: data.text,
      sourceKind: data.sourceKind,
      config: data.config,
    })));
    postMessage({ id: data.id, response });
  } catch (error) {
    postMessage({ type: "error", message: String(error) });
  }
});
