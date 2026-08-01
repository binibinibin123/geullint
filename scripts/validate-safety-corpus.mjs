import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export function normalizeCorpusText(text) {
  return text.trim().replace(/\s+/gu, " ");
}

export function characterTrigrams(text) {
  const characters = Array.from(normalizeCorpusText(text));
  if (characters.length < 3) {
    return new Set(characters.length === 0 ? [] : [characters.join("")]);
  }
  const trigrams = new Set();
  for (let index = 0; index <= characters.length - 3; index += 1) {
    trigrams.add(characters.slice(index, index + 3).join(""));
  }
  return trigrams;
}

export function jaccardSimilarity(left, right) {
  if (left.size === 0 && right.size === 0) {
    return 1;
  }
  let intersection = 0;
  for (const value of left) {
    if (right.has(value)) intersection += 1;
  }
  return intersection / (left.size + right.size - intersection);
}

function isRecord(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function positiveInteger(value) {
  return Number.isInteger(value) && value > 0;
}

function nonNegativeInteger(value) {
  return Number.isInteger(value) && value >= 0;
}

function validatePolicy(policy, errors) {
  if (!isRecord(policy)) {
    errors.push("policy must be a JSON object");
    return false;
  }
  if (policy.schemaVersion !== 1) {
    errors.push("policy schemaVersion must be 1");
  }
  if (policy.exactCaseCount !== undefined && !positiveInteger(policy.exactCaseCount)) {
    errors.push("policy exactCaseCount must be a positive integer");
  }
  if (policy.minCaseCount !== undefined && !positiveInteger(policy.minCaseCount)) {
    errors.push("policy minCaseCount must be a positive integer");
  }
  if (policy.exactCaseCount !== undefined && policy.minCaseCount !== undefined) {
    errors.push("policy must not combine exactCaseCount and minCaseCount");
  }
  for (const field of ["minErrorCases", "minNormalCases", "minCasesPerGenre", "minProfiles"]) {
    if (policy[field] !== undefined && !positiveInteger(policy[field])) {
      errors.push(`policy ${field} must be a positive integer`);
    }
  }
  if (
    policy.maxNormalizedDuplicateCount !== undefined &&
    !nonNegativeInteger(policy.maxNormalizedDuplicateCount)
  ) {
    errors.push("policy maxNormalizedDuplicateCount must be a non-negative integer");
  }
  if (
    policy.maxChar3GramJaccardSimilarity !== undefined &&
    (!Number.isFinite(policy.maxChar3GramJaccardSimilarity) ||
      policy.maxChar3GramJaccardSimilarity < 0 ||
      policy.maxChar3GramJaccardSimilarity > 1)
  ) {
    errors.push("policy maxChar3GramJaccardSimilarity must be between 0 and 1");
  }
  if (policy.minTextLength !== undefined && !positiveInteger(policy.minTextLength)) {
    errors.push("policy minTextLength must be a positive integer");
  }
  for (const field of ["requiredGenres", "requiredSourceKinds", "requiredRuleIds"]) {
    if (
      policy[field] !== undefined &&
      (!Array.isArray(policy[field]) ||
        policy[field].some((value) => typeof value !== "string" || value.trim() === ""))
    ) {
      errors.push(`policy ${field} must contain non-empty strings`);
    }
  }
  if (policy.highRiskRuleMinimums !== undefined) {
    if (
      !isRecord(policy.highRiskRuleMinimums) ||
      Object.entries(policy.highRiskRuleMinimums).some(
        ([ruleId, minimum]) => ruleId.trim() === "" || !positiveInteger(minimum),
      )
    ) {
      errors.push("policy highRiskRuleMinimums must map rule IDs to positive integers");
    }
  }
  if (policy.minimumProfileCounts !== undefined) {
    if (
      !isRecord(policy.minimumProfileCounts) ||
      Object.entries(policy.minimumProfileCounts).some(
        ([profile, minimum]) => profile.trim() === "" || !positiveInteger(minimum),
      )
    ) {
      errors.push("policy minimumProfileCounts must map profiles to positive integers");
    }
  }
  return errors.length === 0;
}

function parseJsonLines(jsonl, errors) {
  const cases = [];
  for (const [index, line] of jsonl.split(/\r?\n/u).entries()) {
    if (line.trim() === "") continue;
    try {
      const value = JSON.parse(line);
      if (!isRecord(value)) {
        errors.push(`line ${index + 1}: case must be a JSON object`);
      } else {
        cases.push({ line: index + 1, value });
      }
    } catch (error) {
      errors.push(`line ${index + 1}: invalid JSON (${error.message})`);
    }
  }
  if (cases.length === 0) errors.push("corpus must contain at least one case");
  return cases;
}

function occurrenceStarts(text, original) {
  const starts = [];
  let offset = 0;
  for (const character of text) {
    if (text.startsWith(original, offset)) starts.push(offset);
    offset += character.length;
  }
  return starts;
}

function stringIndexForByteOffset(text, byteOffset) {
  let bytes = 0;
  let index = 0;
  for (const character of text) {
    if (bytes === byteOffset) return index;
    bytes += Buffer.byteLength(character);
    index += character.length;
    if (bytes > byteOffset) return null;
  }
  return bytes === byteOffset ? index : null;
}

function validateDiagnostic({ diagnostic, entry, index, knownRuleIds, errors }) {
  const prefix = `line ${entry.line} diagnostic ${index + 1}`;
  if (!isRecord(diagnostic)) {
    errors.push(`${prefix}: expectedDiagnostic must be an object`);
    return null;
  }
  if (typeof diagnostic.ruleId !== "string" || diagnostic.ruleId.trim() === "") {
    errors.push(`${prefix}: ruleId must be a non-empty string`);
  } else if (knownRuleIds && !knownRuleIds.has(diagnostic.ruleId)) {
    errors.push(`${prefix}: unknown ruleId \`${diagnostic.ruleId}\``);
  }
  if (typeof diagnostic.original !== "string" || diagnostic.original === "") {
    errors.push(`${prefix}: original must be a non-empty string`);
    return null;
  }
  const suggestionsAreInvalid =
    !Array.isArray(diagnostic.suggestions) ||
    diagnostic.suggestions.length === 0 ||
    diagnostic.suggestions.some((suggestion) => typeof suggestion !== "string");
  if (suggestionsAreInvalid) {
    errors.push(`${prefix}: suggestions must contain strings`);
  }

  let editStart = null;
  let editEnd = null;
  if (diagnostic.range !== undefined) {
    const { range } = diagnostic;
    const byteLength = Buffer.byteLength(entry.value.text);
    if (
      !isRecord(range) ||
      !nonNegativeInteger(range.start) ||
      !nonNegativeInteger(range.end) ||
      range.start > range.end ||
      range.end > byteLength
    ) {
      errors.push(`${prefix}: range must be a valid UTF-8 byte range`);
    } else {
      const stringStart = stringIndexForByteOffset(entry.value.text, range.start);
      const stringEnd = stringIndexForByteOffset(entry.value.text, range.end);
      if (stringStart === null || stringEnd === null) {
        errors.push(`${prefix}: range must use UTF-8 character boundaries`);
      } else if (entry.value.text.slice(stringStart, stringEnd) !== diagnostic.original) {
        errors.push(`${prefix}: range does not equal original`);
      } else {
        editStart = stringStart;
        editEnd = stringEnd;
      }
    }
  } else {
    const starts = occurrenceStarts(entry.value.text, diagnostic.original);
    if (starts.length !== 1) {
      errors.push(`${prefix}: original must occur exactly once (found ${starts.length})`);
    } else {
      editStart = starts[0];
      editEnd = starts[0] + diagnostic.original.length;
    }
  }
  if (editStart === null || editEnd === null || suggestionsAreInvalid) {
    return null;
  }
  return {
    start: editStart,
    end: editEnd,
    replacement: diagnostic.suggestions[0],
    ruleId: diagnostic.ruleId,
  };
}

function applyAnnotatedFixes(text, edits) {
  let fixed = text;
  const sorted = edits.toSorted(
    (left, right) => right.start - left.start || right.end - left.end || left.ruleId.localeCompare(right.ruleId),
  );
  let nextStart = text.length;
  for (const edit of sorted) {
    if (edit.end > nextStart) return null;
    fixed = `${fixed.slice(0, edit.start)}${edit.replacement}${fixed.slice(edit.end)}`;
    nextStart = edit.start;
  }
  return fixed;
}

function increment(map, key) {
  map.set(key, (map.get(key) ?? 0) + 1);
}

export function validateSafetyCorpus({ jsonl, policy, knownRuleIds } = {}) {
  const errors = [];
  if (typeof jsonl !== "string") {
    return { valid: false, errors: ["jsonl must be a string"], summary: null };
  }
  if (!validatePolicy(policy, errors)) {
    return { valid: false, errors, summary: null };
  }
  const known =
    knownRuleIds === undefined
      ? undefined
      : knownRuleIds instanceof Set
        ? knownRuleIds
        : new Set(knownRuleIds);
  const entries = parseJsonLines(jsonl, errors);
  const seenIds = new Map();
  const seenTexts = new Map();
  const duplicateTexts = [];
  const genreCounts = new Map();
  const sourceKindCounts = new Map();
  const profileCounts = new Map();
  const positiveRuleCounts = new Map();
  let errorCases = 0;
  let normalCases = 0;
  let normalizedDuplicateCount = 0;

  for (const entry of entries) {
    const { value } = entry;
    const prefix = `line ${entry.line}`;
    if (typeof value.id !== "string" || value.id.trim() === "") {
      errors.push(`${prefix}: id must be a non-empty string`);
    } else if (seenIds.has(value.id.trim())) {
      errors.push(`${prefix}: duplicate id \`${value.id.trim()}\``);
    } else {
      seenIds.set(value.id.trim(), entry.line);
    }
    if (typeof value.text !== "string" || value.text.trim() === "") {
      errors.push(`${prefix}: text must be a non-empty string`);
      continue;
    }
    const normalizedText = normalizeCorpusText(value.text);
    if (seenTexts.has(normalizedText)) {
      normalizedDuplicateCount += 1;
      duplicateTexts.push({
        duplicate: {
          id: typeof value.id === "string" && value.id.trim() !== "" ? value.id.trim() : "<missing>",
          line: entry.line,
        },
        original: seenTexts.get(normalizedText),
      });
    } else {
      seenTexts.set(normalizedText, {
        id: typeof value.id === "string" && value.id.trim() !== "" ? value.id.trim() : "<missing>",
        line: entry.line,
      });
    }
    if (
      policy.minTextLength !== undefined &&
      Array.from(normalizedText).length < policy.minTextLength
    ) {
      errors.push(`${prefix}: text is shorter than minTextLength ${policy.minTextLength}`);
    }

    for (const field of ["genre", "sourceKind", "profile", "provenanceId"]) {
      if (typeof value[field] !== "string" || value[field].trim() === "") {
        errors.push(`${prefix}: ${field} must be a non-empty string`);
      }
    }
    if (typeof value.genre === "string" && value.genre.trim() !== "") {
      increment(genreCounts, value.genre);
    }
    if (typeof value.sourceKind === "string" && value.sourceKind.trim() !== "") {
      increment(sourceKindCounts, value.sourceKind);
    }
    if (typeof value.profile === "string" && value.profile.trim() !== "") {
      increment(profileCounts, value.profile);
    }

    if (!Array.isArray(value.expectedDiagnostics)) {
      errors.push(`${prefix}: expectedDiagnostics must be an array`);
      value.expectedDiagnostics = [];
    }
    const diagnostics = value.expectedDiagnostics;
    if (value.caseType === "normal") {
      normalCases += 1;
      if (diagnostics.length !== 0) {
        errors.push(`${prefix}: normal case must not contain expected diagnostics`);
      }
    } else if (value.caseType === "error") {
      errorCases += 1;
      if (diagnostics.length === 0) {
        errors.push(`${prefix}: error case requires at least one expected diagnostic`);
      }
    } else {
      errors.push(`${prefix}: caseType must be \`error\` or \`normal\``);
    }

    const edits = diagnostics
      .map((diagnostic, index) =>
        validateDiagnostic({ diagnostic, entry, index, knownRuleIds: known, errors }),
      )
      .filter(Boolean);
    for (const diagnostic of diagnostics) {
      if (typeof diagnostic?.ruleId === "string" && diagnostic.ruleId.trim() !== "") {
        increment(positiveRuleCounts, diagnostic.ruleId);
      }
    }

    if (typeof value.expectedFixedText !== "string") {
      errors.push(`${prefix}: expectedFixedText must be a string`);
    } else if (value.caseType === "normal" && value.expectedFixedText !== value.text) {
      errors.push(`${prefix}: normal case expectedFixedText must equal text`);
    } else if (value.caseType === "error" && value.expectedFixedText !== value.text) {
      const annotatedFixedText = applyAnnotatedFixes(value.text, edits);
      if (annotatedFixedText === null || value.expectedFixedText !== annotatedFixedText) {
        errors.push(
          `${prefix}: changed expectedFixedText must apply the first annotated suggestions exactly`,
        );
      }
    }
  }

  const caseCount = entries.length;
  if (policy.exactCaseCount !== undefined && caseCount !== policy.exactCaseCount) {
    errors.push(`case count ${caseCount} does not equal exactCaseCount ${policy.exactCaseCount}`);
  }
  if (policy.minCaseCount !== undefined && caseCount < policy.minCaseCount) {
    errors.push(`case count ${caseCount} is below minCaseCount ${policy.minCaseCount}`);
  }
  if (policy.minErrorCases !== undefined && errorCases < policy.minErrorCases) {
    errors.push(`error case count ${errorCases} is below minErrorCases ${policy.minErrorCases}`);
  }
  if (policy.minNormalCases !== undefined && normalCases < policy.minNormalCases) {
    errors.push(`normal case count ${normalCases} is below minNormalCases ${policy.minNormalCases}`);
  }
  for (const genre of policy.requiredGenres ?? []) {
    const actual = genreCounts.get(genre) ?? 0;
    if (actual === 0) errors.push(`required genre \`${genre}\` is missing`);
    if (policy.minCasesPerGenre !== undefined && actual < policy.minCasesPerGenre) {
      errors.push(
        `genre \`${genre}\` has ${actual} cases; requires at least ${policy.minCasesPerGenre}`,
      );
    }
  }
  for (const sourceKind of policy.requiredSourceKinds ?? []) {
    if (!sourceKindCounts.has(sourceKind)) {
      errors.push(`required sourceKind \`${sourceKind}\` is missing`);
    }
  }
  if (policy.minProfiles !== undefined && profileCounts.size < policy.minProfiles) {
    errors.push(`profile count ${profileCounts.size} is below minProfiles ${policy.minProfiles}`);
  }
  for (const [profile, minimum] of Object.entries(policy.minimumProfileCounts ?? {})) {
    const actual = profileCounts.get(profile) ?? 0;
    if (actual < minimum) {
      errors.push(`profile \`${profile}\` has ${actual} cases; requires at least ${minimum}`);
    }
  }
  const maximumDuplicateCount = policy.maxNormalizedDuplicateCount ?? 0;
  if (normalizedDuplicateCount > maximumDuplicateCount) {
    for (const duplicate of duplicateTexts) {
      errors.push(
        `case \`${duplicate.duplicate.id}\` (line ${duplicate.duplicate.line}) has duplicate normalized text; first seen in case \`${duplicate.original.id}\` (line ${duplicate.original.line})`,
      );
    }
    errors.push(
      `normalized duplicate count ${normalizedDuplicateCount} exceeds ${maximumDuplicateCount}`,
    );
  }
  for (const ruleId of policy.requiredRuleIds ?? []) {
    if ((positiveRuleCounts.get(ruleId) ?? 0) === 0) {
      errors.push(`required ruleId \`${ruleId}\` has no positive case`);
    }
  }
  for (const [ruleId, minimum] of Object.entries(policy.highRiskRuleMinimums ?? {})) {
    const actual = positiveRuleCounts.get(ruleId) ?? 0;
    if (actual < minimum) {
      errors.push(
        `high-risk positive count for \`${ruleId}\` is ${actual}; requires at least ${minimum}`,
      );
    }
  }

  if (policy.maxChar3GramJaccardSimilarity !== undefined) {
    const uniqueTexts = [...seenTexts.entries()].map(([text, identity]) => ({ identity, text }));
    const trigrams = uniqueTexts.map(({ text }) => characterTrigrams(text));
    for (let left = 0; left < uniqueTexts.length; left += 1) {
      for (let right = left + 1; right < uniqueTexts.length; right += 1) {
        const similarity = jaccardSimilarity(trigrams[left], trigrams[right]);
        if (similarity > policy.maxChar3GramJaccardSimilarity) {
          const leftIdentity = uniqueTexts[left].identity;
          const rightIdentity = uniqueTexts[right].identity;
          errors.push(
            `case \`${leftIdentity.id}\` (line ${leftIdentity.line}) and case \`${rightIdentity.id}\` (line ${rightIdentity.line}) have 3-gram Jaccard similarity ${similarity.toFixed(3)}, above ${policy.maxChar3GramJaccardSimilarity}`,
          );
        }
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    summary: {
      cases: caseCount,
      errorCases,
      normalCases,
      genres: genreCounts.size,
      sourceKinds: sourceKindCounts.size,
      profiles: profileCounts.size,
      normalizedDuplicateCount,
    },
  };
}

export function readKnownRuleIds(cliPath) {
  const output = spawnSync(cliPath, ["rules", "--format", "json"], {
    encoding: "utf8",
    windowsHide: true,
  });
  if (output.error) throw output.error;
  if (output.status !== 0) {
    throw new Error(`geullint rules failed: ${output.stderr.trim()}`);
  }
  const catalog = JSON.parse(output.stdout);
  if (!Array.isArray(catalog.rules)) {
    throw new Error("geullint rules output does not contain a rules array");
  }
  return new Set(catalog.rules.map((rule) => rule.id));
}

function parseArguments(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!["--corpus", "--policy", "--cli"].includes(flag) || value === undefined) {
      throw new Error(
        "usage: node scripts/validate-safety-corpus.mjs --corpus PATH --policy PATH [--cli PATH]",
      );
    }
    options[flag.slice(2)] = value;
  }
  if (!options.corpus || !options.policy) {
    throw new Error("--corpus and --policy are required");
  }
  return options;
}

function main() {
  try {
    const options = parseArguments(process.argv.slice(2));
    const result = validateSafetyCorpus({
      jsonl: readFileSync(options.corpus, "utf8"),
      policy: JSON.parse(readFileSync(options.policy, "utf8")),
      knownRuleIds: options.cli ? readKnownRuleIds(options.cli) : undefined,
    });
    if (!result.valid) {
      for (const error of result.errors) console.error(`safety corpus: ${error}`);
      process.exitCode = 1;
      return;
    }
    console.log(JSON.stringify(result.summary, null, 2));
  } catch (error) {
    console.error(`safety corpus: ${error.message}`);
    process.exitCode = 2;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
