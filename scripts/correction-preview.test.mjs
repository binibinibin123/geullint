import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import test from "node:test";
import { applySuggestedFixes } from "../apps/playground/corrections.js";

function rangeOf(text, needle) {
  const characterStart = text.indexOf(needle);
  assert.notEqual(characterStart, -1, `${needle} must appear in the source`);
  return {
    start: Buffer.byteLength(text.slice(0, characterStart), "utf8"),
    end: Buffer.byteLength(text.slice(0, characterStart + needle.length), "utf8"),
  };
}

test("keeps review suggestions out of the default correction preview", () => {
  const source = "감사해용 웬만하면 돼게 할려고 하였다";
  const diagnostics = [
    {
      original: "감사해용",
      range: rangeOf(source, "감사해용"),
      suggestions: ["감사해요"],
      safeFix: false,
    },
    {
      original: "돼게",
      range: rangeOf(source, "돼게"),
      suggestions: ["되게"],
      safeFix: true,
    },
    {
      original: "할려고",
      range: rangeOf(source, "할려고"),
      suggestions: ["하려고"],
      safeFix: true,
    },
  ];

  assert.equal(
    applySuggestedFixes(source, diagnostics),
    "감사해용 웬만하면 되게 하려고 하였다",
  );
  assert.equal(
    applySuggestedFixes(source, diagnostics, { includeReview: true }),
    "감사해요 웬만하면 되게 하려고 하였다",
  );
});
