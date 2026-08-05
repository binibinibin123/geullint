import assert from "node:assert/strict";
import test from "node:test";
import { evaluateGateReport } from "./evaluate-commercial-gate.mjs";
import { RED_TEAM_CASES, evaluateRedTeamResults } from "./red-team-korean.mjs";

test("commercial gate report never turns a failed CLI into a pass", () => {
  const passed = evaluateGateReport({ qualityGate: { passed: true }, cases: 20 }, 0);
  assert.equal(passed.passed, true);
  const failed = evaluateGateReport({ qualityGate: { passed: true }, cases: 20 }, 1);
  assert.equal(failed.passed, false);
});

test("commercial gate report includes auxiliary leakage, review, and parity checks", () => {
  const result = evaluateGateReport(
    { qualityGate: { passed: true }, cases: 20 },
    0,
    {
      leakage: { passed: true },
      reviewQuality: { passed: false },
      parity: { passed: true },
    },
  );
  assert.equal(result.passed, false);
  assert.equal(result.checks.reviewQuality.passed, false);
});

test("red-team fixtures include hard negatives and expected positive families", () => {
  assert.ok(RED_TEAM_CASES.some((fixture) => fixture.expected.length === 0));
  assert.ok(RED_TEAM_CASES.some((fixture) => fixture.expected.includes("spelling.lexical.myeochil")));
  const report = evaluateRedTeamResults([{ id: "ok", passed: true }]);
  assert.equal(report.passed, true);
});
