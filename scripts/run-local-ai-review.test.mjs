import assert from "node:assert/strict";
import test from "node:test";

import { chunkItems } from "./run-local-ai-review.mjs";

test("chunks AI review cases without dropping order or records", () => {
  const cases = [{ id: "a" }, { id: "b" }, { id: "c" }, { id: "d" }, { id: "e" }];
  assert.deepEqual(chunkItems(cases, 2), [[cases[0], cases[1]], [cases[2], cases[3]], [cases[4]]]);
});
