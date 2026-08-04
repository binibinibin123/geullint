export function createHistory(initial, maxStates = 50) {
  const limit = Math.max(2, maxStates);
  const entries = [String(initial ?? "")];
  let cursor = 0;

  return {
    push(value) {
      const next = String(value ?? "");
      if (entries[cursor] === next) return;
      entries.splice(cursor + 1);
      entries.push(next);
      if (entries.length > limit) {
        entries.shift();
      }
      cursor = entries.length - 1;
    },
    current() {
      return entries[cursor];
    },
    undo() {
      if (cursor === 0) return undefined;
      cursor -= 1;
      return entries[cursor];
    },
    redo() {
      if (cursor >= entries.length - 1) return undefined;
      cursor += 1;
      return entries[cursor];
    },
    canUndo() {
      return cursor > 0;
    },
    canRedo() {
      return cursor < entries.length - 1;
    },
    states() {
      return [...entries];
    },
  };
}
