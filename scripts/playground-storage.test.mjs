import assert from "node:assert/strict";
import test from "node:test";

import { createHistory } from "../apps/playground/history.js";

test("history keeps bounded reversible text states without duplicating entries", () => {
  const history = createHistory("첫 문장", 3);
  history.push("둘째 문장");
  history.push("둘째 문장");
  history.push("셋째 문장");
  assert.equal(history.current(), "셋째 문장");
  assert.equal(history.undo(), "둘째 문장");
  assert.equal(history.undo(), "첫 문장");
  assert.equal(history.undo(), undefined);
  history.push("넷째 문장");
  assert.equal(history.current(), "넷째 문장");
});

test("history truncates redo states after a new edit", () => {
  const history = createHistory("a", 4);
  history.push("b");
  history.push("c");
  assert.equal(history.undo(), "b");
  history.push("d");
  assert.equal(history.redo(), undefined);
  assert.deepEqual(history.states(), ["a", "b", "d"]);
});
