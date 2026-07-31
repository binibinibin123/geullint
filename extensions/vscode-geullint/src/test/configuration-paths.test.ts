import assert from "node:assert/strict";
import test from "node:test";

import { createLspConfiguration, firstWorkspaceFolderUri } from "../configuration";
import { resolveWorkspacePaths } from "../configuration-paths";

test("resolves relative rule packs from the workspace root", () => {
  assert.deepEqual(
    resolveWorkspacePaths(
      [".geullint-rules.yaml", "C:/shared/editorial.yaml"],
      "C:/project",
      "win32",
    ),
    ["C:\\project\\.geullint-rules.yaml", "C:/shared/editorial.yaml"],
  );
});

test("resolves overlay and rule-pack paths for every LSP configuration update", () => {
  const values: Record<string, unknown> = {
    profile: "strict",
    userDictionary: ["GeulLint"],
    dictionaryOverlay: ["프로젝트오표기"],
    dictionaryOverlayPaths: [".geullint.overlay"],
    rulePacks: ["rules/editorial.yaml"],
  };
  const configuration = {
    get<T>(name: string, fallback: T): T {
      return (values[name] ?? fallback) as T;
    },
  };

  assert.deepEqual(createLspConfiguration(configuration, "C:/project", "win32"), {
    profile: "strict",
    userDictionary: ["GeulLint"],
    dictionaryOverlay: ["프로젝트오표기"],
    dictionaryOverlayPaths: ["C:\\project\\.geullint.overlay"],
    rulePacks: ["C:\\project\\rules\\editorial.yaml"],
  });
});

test("uses the first workspace folder URI for resource-scoped configuration", () => {
  const first = { uri: "file:///C:/project" };
  assert.equal(
    firstWorkspaceFolderUri([first, { uri: "file:///C:/other-project" }]),
    first.uri,
  );
  assert.equal(firstWorkspaceFolderUri([]), undefined);
});
