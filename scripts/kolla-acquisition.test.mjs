import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import test from "node:test";

import * as acquisition from "./acquire-kolla-v2.mjs";

const { allNoopCases } = acquisition;

test("keeps only KoLLA sentences that every annotator marked noop", () => {
  const cases = allNoopCases([
    "S 목요일이었습니다 .",
    "A -1 -1|||noop|||-NONE-|||REQUIRED|||-NONE-|||0",
    "A -1 -1|||noop|||-NONE-|||REQUIRED|||-NONE-|||1",
    "",
    "S 제 딸이 포타 앞에 있습니다 .",
    "A -1 -1|||noop|||-NONE-|||REQUIRED|||-NONE-|||0",
    "A 2 3|||R:SPELL|||포터|||REQUIRED|||-NONE-|||1",
    "",
  ].join("\n"));

  assert.deepEqual(cases, [
    {
      id: "kolla-v2-noop-1",
      text: "목요일이었습니다.",
      sourceKind: "plain_text",
      expectedRuleIds: [],
    },
  ]);
});

test("exports non-noop M2 edits as a human-reviewable correction queue", () => {
  assert.equal(typeof acquisition.correctionReviewQueue, "function");

  const cases = acquisition.correctionReviewQueue([
    "S 제 딸이 포타 앞에 있습니다 .",
    "A 2 3|||R:SPELL|||포터|||REQUIRED|||-NONE-|||0",
    "A 2 3|||R:SPELL|||포터|||REQUIRED|||-NONE-|||1",
    "",
    "S 목요일이었습니다 .",
    "A -1 -1|||noop|||-NONE-|||REQUIRED|||-NONE-|||0",
    "",
  ].join("\n"));

  assert.deepEqual(cases, [
    {
      id: "kolla-v2-review-1",
      text: "제 딸이 포타 앞에 있습니다.",
      sourceTokens: ["제", "딸이", "포타", "앞에", "있습니다", "."],
      references: [
        {
          annotator: "0",
          edits: [
            {
              startToken: 2,
              endToken: 3,
              category: "R:SPELL",
              correction: "포터",
            },
          ],
        },
        {
          annotator: "1",
          edits: [
            {
              startToken: 2,
              endToken: 3,
              category: "R:SPELL",
              correction: "포터",
            },
          ],
        },
      ],
    },
  ]);
});

test("requires explicit acceptance before downloading GPL-licensed data", () => {
  const result = spawnSync(process.execPath, ["scripts/acquire-kolla-v2.mjs"], {
    encoding: "utf8",
  });

  assert.equal(result.status, 2);
  assert.match(result.stderr, /--accept-gpl-3\.0-or-later/);
});
