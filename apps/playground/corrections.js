const textEncoder = new TextEncoder();

function stringIndexForByte(text, byteOffset) {
  let bytes = 0;
  let index = 0;

  for (const character of text) {
    if (bytes === byteOffset) return index;
    bytes += textEncoder.encode(character).length;
    index += character.length;
    if (bytes > byteOffset) return null;
  }

  return bytes === byteOffset ? index : null;
}

export function replaceUtf8Range(text, range, replacement) {
  const start = stringIndexForByte(text, range.start);
  const end = stringIndexForByte(text, range.end);
  if (start === null || end === null || start > end) return text;
  return `${text.slice(0, start)}${replacement}${text.slice(end)}`;
}

function sourceForRange(text, range) {
  const start = stringIndexForByte(text, range.start);
  const end = stringIndexForByte(text, range.end);
  return start === null || end === null || start > end ? null : text.slice(start, end);
}

export function applySuggestedFixes(text, diagnostics, { includeReview = false } = {}) {
  const candidates = diagnostics
    .filter((diagnostic) => diagnostic.suggestions?.[0])
    .filter((diagnostic) => diagnostic.safeFix || includeReview)
    .filter((diagnostic) => sourceForRange(text, diagnostic.range) === diagnostic.original)
    .sort((left, right) => (
      left.range.start - right.range.start
      || left.range.end - right.range.end
      || Number(right.safeFix) - Number(left.safeFix)
      || left.ruleId.localeCompare(right.ruleId)
    ));

  const accepted = [];
  let previousEnd = 0;
  for (const candidate of candidates) {
    if (candidate.range.start >= previousEnd) {
      accepted.push(candidate);
      previousEnd = candidate.range.end;
    }
  }

  return accepted
    .toReversed()
    .reduce(
      (corrected, diagnostic) => replaceUtf8Range(
        corrected,
        diagnostic.range,
        diagnostic.suggestions[0],
      ),
      text,
    );
}
