import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { resolveServerCommand } from "../server-path";

test("uses an explicitly configured server path first", () => {
  const command = resolveServerCommand({
    extensionPath: "C:/extension",
    configuredPath: "C:/tools/geullint-lsp.exe",
    platform: "win32",
    arch: "x64",
    pathExists: () => true,
  });

  assert.equal(command, "C:/tools/geullint-lsp.exe");
});

test("uses the matching bundled release binary when available", () => {
  const command = resolveServerCommand({
    extensionPath: "C:/extension",
    configuredPath: "",
    platform: "darwin",
    arch: "arm64",
    pathExists: (candidate) => candidate.endsWith("geullint-lsp"),
  });

  assert.equal(
    command,
    path.join("C:/extension", "server", "darwin-arm64", "geullint-lsp"),
  );
});

test("falls back to the PATH command for local development", () => {
  const command = resolveServerCommand({
    extensionPath: "C:/extension",
    configuredPath: "",
    platform: "linux",
    arch: "x64",
    pathExists: () => false,
  });

  assert.equal(command, "geullint-lsp");
});

test("declares profile, personal dictionary, local overlay path, and rule pack settings", () => {
  const manifest = JSON.parse(fs.readFileSync("package.json", "utf8")) as {
    contributes: {
      configuration: {
        properties: Record<string, { default?: unknown; enum?: unknown[] }>;
      };
    };
  };
  const properties = manifest.contributes.configuration.properties;

  assert.deepEqual(properties["geullint.profile"].enum, [
    "default",
    "strict",
    "editorial",
  ]);
  assert.equal(properties["geullint.profile"].default, "default");
  assert.deepEqual(properties["geullint.userDictionary"].default, []);
  assert.deepEqual(properties["geullint.dictionaryOverlay"].default, []);
  assert.deepEqual(properties["geullint.dictionaryOverlayPaths"].default, []);
  assert.deepEqual(properties["geullint.rulePacks"].default, []);
});
