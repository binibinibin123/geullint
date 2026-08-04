import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { planDownloads, validateSourceManifest } from "./acquire-training-data.mjs";

test("checked-in corpus sources keep the NIKL holdout request-only", async () => {
  const manifest = JSON.parse(await readFile("data/sources.json", "utf8"));
  validateSourceManifest(manifest);

  const nikl = manifest.sources.find((source) => source.id === "nikl-spelling-correction-2021");
  assert.ok(nikl);
  assert.equal(nikl.access, "manual_authorization");
  assert.equal(nikl.redistributable, false);
  assert.equal(nikl.sha256, null);
  assert.equal(nikl.role, "independent-release-holdout-pending");
  assert.equal(planDownloads(manifest).some((source) => source.id === nikl.id), false);
});
