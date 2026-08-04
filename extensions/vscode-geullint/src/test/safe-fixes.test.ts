import assert from "node:assert/strict";
import test from "node:test";

import { collectSafeFixEdits } from "../safe-fixes";

test("collects only GeulLint safe replacements for Fix All", () => {
  const edits = collectSafeFixEdits([
    {
      source: "geullint",
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 2 } },
      data: { safeFix: true, replacement: "며칠" },
    },
    {
      source: "geullint",
      range: { start: { line: 1, character: 0 }, end: { line: 1, character: 2 } },
      data: { safeFix: false, replacement: "검토" },
    },
    {
      source: "other",
      range: { start: { line: 2, character: 0 }, end: { line: 2, character: 2 } },
      data: { safeFix: true, replacement: "무시" },
    },
  ]);

  assert.deepEqual(edits, [
    {
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 2 } },
      newText: "며칠",
    },
  ]);
});
