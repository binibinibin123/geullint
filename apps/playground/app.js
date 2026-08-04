import { applyLocale } from "./i18n.js";
import { replaceUtf8Range } from "./corrections.js";

const editor = document.querySelector("#editor");
const profile = document.querySelector("#profile");
const sourceKind = document.querySelector("#source-kind");
const language = document.querySelector("#language");
const scanButton = document.querySelector("#scan");
const sampleButton = document.querySelector("#sample");
const results = document.querySelector("#results");
const findingCount = document.querySelector("#finding-count");
const characterCount = document.querySelector("#character-count");
const correctionPanel = document.querySelector("#correction-panel");
const correctedOutput = document.querySelector("#corrected-output");
const correctionStatus = document.querySelector("#correction-status");
const copyCorrection = document.querySelector("#copy-correction");
const applyCorrection = document.querySelector("#apply-correction");
const undoCorrection = document.querySelector("#undo-correction");
const includeReviewCorrections = document.querySelector("#include-review-corrections");
const ruleSearch = document.querySelector("#rule-search");
const ruleList = document.querySelector("#rule-list");
const ruleCount = document.querySelector("#rule-count");

const samples = [
  "안녕하세용 감사해용 왠만하면 돼게 할려고 하였다.",
  "오늘이 몇일이지? 일이 되서 끝낼수 있는지 확인했다.",
  "회의 자료 데이타를 확인하고 올것 같다고 말했다.",
  "결과를 알수없다. 만난적 있는 사람에게 물었다.",
];
const worker = new Worker("./worker.js", { type: "module" });
let requestNumber = 0;
let sampleIndex = 0;
let engineReady = false;
let indexedRules = [];
let currentCopy;
let correctionState = "correctionLoading";
let requestedText = "";
let latestCorrection;
let undoText;
const initialText = editor.value;

export function createRuleIndex(catalog) {
  return catalog.rules.map((rule) => ({
    rule,
    searchable: [
      rule.id,
      rule.title,
      rule.description,
      rule.category,
      ...rule.incorrectExamples,
      ...rule.correctExamples,
    ].join(" ").toLocaleLowerCase(),
  }));
}

function updateCharacterCount() {
  characterCount.textContent = `${[...editor.value].length}자`;
}

function setBusy(isBusy) {
  scanButton.disabled = isBusy;
  results.setAttribute("aria-busy", String(isBusy));
}

function setCorrectionState(state) {
  correctionState = state;
  correctionPanel.dataset.state = state;
  correctionStatus.textContent = currentCopy?.[state] || "";
}

function prepareCorrection() {
  latestCorrection = undefined;
  correctedOutput.value = "";
  correctedOutput.setAttribute("aria-busy", "true");
  copyCorrection.disabled = true;
  applyCorrection.disabled = true;
  setCorrectionState("correctionLoading");
}

function invalidateCorrection() {
  latestCorrection = undefined;
  correctedOutput.value = "";
  correctedOutput.setAttribute("aria-busy", "false");
  copyCorrection.disabled = true;
  applyCorrection.disabled = true;
  setCorrectionState("correctionNeedsScan");
}

function rememberBeforeOverwrite() {
  undoText = editor.value;
  undoCorrection.disabled = false;
}

function renderCorrection(originalText, fixedText, reviewFixedText, diagnostics) {
  latestCorrection = { originalText, fixedText, reviewFixedText, diagnostics };
  const includesReview = includeReviewCorrections.checked
    && diagnostics.some((diagnostic) => !diagnostic.safeFix && diagnostic.suggestions?.[0]);
  const previewText = includesReview
    ? reviewFixedText
    : fixedText;
  correctedOutput.value = previewText;
  correctedOutput.setAttribute("aria-busy", "false");
  const changed = previewText !== originalText;
  const state = changed
    ? includesReview
      ? "correctionReviewApplied"
      : "correctionApplied"
    : diagnostics.length > 0
      ? "correctionReview"
      : "correctionUnchanged";
  setCorrectionState(state);
  copyCorrection.disabled = previewText.length === 0;
  applyCorrection.disabled = !changed;
}

function failCorrection() {
  correctedOutput.value = "";
  correctedOutput.setAttribute("aria-busy", "false");
  copyCorrection.disabled = true;
  applyCorrection.disabled = true;
  setCorrectionState("correctionError");
}

function setMessage(className, text) {
  results.replaceChildren();
  const message = document.createElement("p");
  message.className = className;
  message.textContent = text;
  results.append(message);
}

function renderDiagnostics(diagnostics) {
  results.replaceChildren();
  findingCount.textContent = `${diagnostics.length}건 발견`;
  if (diagnostics.length === 0) {
    setMessage(
      "empty",
      editor.value.trim()
        ? (currentCopy?.noSupportedFindings || "현재 프로필에서 고칠 표현을 찾지 못했습니다.")
        : (currentCopy?.emptyInput || "검사할 문장을 입력하세요."),
    );
    return;
  }

  for (const diagnostic of diagnostics) {
    const finding = document.createElement("article");
    finding.className = "finding";
    const severity = document.createElement("span");
    severity.className = `severity severity-${diagnostic.severity}`;
    severity.textContent = diagnostic.severity;
    const copy = document.createElement("div");
    const rule = document.createElement("p");
    rule.className = "finding-rule";
    rule.textContent = diagnostic.ruleId;
    const message = document.createElement("p");
    message.className = "finding-message";
    message.textContent = diagnostic.message;
    copy.append(rule, message);
    finding.append(severity, copy);
    if (diagnostic.suggestions?.[0]) {
      const apply = document.createElement("button");
      apply.className = "suggestion";
      apply.type = "button";
      apply.textContent = `${diagnostic.original} → ${diagnostic.suggestions[0]}`;
      apply.addEventListener("click", () => {
        rememberBeforeOverwrite();
        editor.value = replaceUtf8Range(
          editor.value,
          diagnostic.range,
          diagnostic.suggestions[0],
        );
        updateCharacterCount();
        scan();
      });
      finding.append(apply);
    }
    results.append(finding);
  }
}

function renderRuleList() {
  const query = ruleSearch.value.trim().toLocaleLowerCase();
  const matches = indexedRules
    .filter((entry) => !query || entry.searchable.includes(query));
  ruleCount.textContent = query
    ? `${matches.length}${indexedRules.length > matches.length ? "+" : ""} / ${indexedRules.length}`
    : `${indexedRules.length} rules`;
  ruleList.replaceChildren();
  for (const { rule } of matches) {
    const card = document.createElement("details");
    card.className = "rule-card";
    const summary = document.createElement("summary");
    const title = document.createElement("strong");
    title.textContent = rule.title;
    const id = document.createElement("code");
    id.textContent = rule.id;
    summary.append(title, id);
    const description = document.createElement("p");
    description.textContent = rule.description;
    const example = document.createElement("p");
    example.className = "rule-example";
    example.textContent = `${rule.incorrectExamples[0]} → ${rule.correctExamples[0]}`;
    const meta = document.createElement("small");
    meta.textContent = `${rule.category} · ${rule.confidence} · ${rule.defaultEnabled ? "default" : "opt-in"}`;
    card.append(summary, description, example, meta);
    ruleList.append(card);
  }
}

function scan() {
  if (!engineReady) {
    setMessage("loading", "오프라인 엔진을 준비하고 있습니다…");
    prepareCorrection();
    return;
  }
  const id = ++requestNumber;
  requestedText = editor.value;
  setBusy(true);
  prepareCorrection();
  worker.postMessage({
    id,
    text: requestedText,
    sourceKind: sourceKind.value,
    config: { profile: profile.value },
    includeReviewFixes: includeReviewCorrections.checked,
  });
}

worker.addEventListener("message", ({ data }) => {
  if (data.type === "ready") {
    engineReady = true;
    indexedRules = createRuleIndex(data.catalog);
    renderRuleList();
    findingCount.textContent = "엔진 준비됨";
    setBusy(false);
    scan();
    return;
  }
  if (data.type === "error") {
    setBusy(false);
    failCorrection();
    findingCount.textContent = "엔진 오류";
    setMessage("error", `점검기를 시작하지 못했습니다: ${data.message}`);
    return;
  }
  if (data.id !== requestNumber) return;
  if (editor.value !== requestedText) {
    setBusy(false);
    invalidateCorrection();
    return;
  }
  setBusy(false);
  renderCorrection(
    requestedText,
    data.response.fixedText,
    data.response.reviewFixedText,
    data.response.diagnostics,
  );
  renderDiagnostics(data.response.diagnostics);
});

editor.addEventListener("input", () => {
  updateCharacterCount();
  invalidateCorrection();
});
editor.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") scan();
});
sampleButton.addEventListener("click", () => {
  sampleIndex = (sampleIndex + 1) % samples.length;
  rememberBeforeOverwrite();
  editor.value = samples[sampleIndex];
  updateCharacterCount();
  scan();
});
scanButton.addEventListener("click", scan);
copyCorrection.addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(correctedOutput.value);
    setCorrectionState("copySucceeded");
  } catch {
    correctedOutput.focus();
    correctedOutput.select();
    setCorrectionState("copyFailed");
  }
});
applyCorrection.addEventListener("click", () => {
  rememberBeforeOverwrite();
  editor.value = correctedOutput.value;
  updateCharacterCount();
  editor.focus();
  scan();
});
undoCorrection.addEventListener("click", () => {
  if (undoText === undefined) return;
  editor.value = undoText;
  undoText = undefined;
  undoCorrection.disabled = true;
  updateCharacterCount();
  editor.focus();
  scan();
});
includeReviewCorrections.addEventListener("change", () => {
  scan();
});
ruleSearch.addEventListener("input", renderRuleList);
language.addEventListener("change", () => {
  currentCopy = applyLocale(language.value);
  setCorrectionState(correctionState);
  try {
    localStorage.setItem("geullint-locale", language.value);
  } catch {
    // Private browsing may disable storage; translation still works.
  }
});

try {
  language.value = localStorage.getItem("geullint-locale") || "ko";
} catch {
  language.value = "ko";
}
currentCopy = applyLocale(language.value);
setCorrectionState(correctionState);
updateCharacterCount();
if ("serviceWorker" in navigator) {
  navigator.serviceWorker.register("./sw.js", { scope: "./" }).catch(() => {
    // The app remains usable when opened from a local file or a host without SW support.
  });
}
window.addEventListener("beforeunload", (event) => {
  if (editor.value !== initialText) {
    event.preventDefault();
    event.returnValue = "";
  }
});
