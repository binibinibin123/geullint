import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const BENCHMARK_SIZES = Object.freeze({
  "1kb": 1_024,
  "100kb": 100 * 1_024,
  "1mb": 1_024 * 1_024,
});

const FIXTURE_KINDS = Object.freeze([
  { id: "plain", sourceKind: "plain_text" },
  { id: "markdown", sourceKind: "markdown" },
  { id: "typescript", sourceKind: "typescript" },
]);

const GENRES = Object.freeze([
  "일상 대화",
  "업무 보고",
  "제품 안내",
  "기술 문서",
  "교육 자료",
  "여행 기록",
  "문화 해설",
  "과학 기사",
  "공공 안내",
  "서평",
]);

const SUBJECTS = Object.freeze([
  "연구팀은",
  "편집자는",
  "도서관은",
  "개발자는",
  "학생들은",
  "지역 주민은",
  "기획자는",
  "상담원은",
  "관람객은",
  "운영진은",
  "기자는",
  "독자는",
]);

const ACTIONS = Object.freeze([
  "자료의 출처를 다시 확인했다",
  "핵심 내용을 간결하게 정리했다",
  "변경된 일정을 참가자에게 알렸다",
  "측정 결과를 표와 문장으로 설명했다",
  "서로 다른 의견을 차분히 비교했다",
  "안전 수칙을 읽기 쉬운 표현으로 고쳤다",
  "회의에서 결정된 항목을 기록했다",
  "낯선 용어에 짧은 해설을 덧붙였다",
  "현장의 질문을 빠짐없이 모았다",
  "완성된 문서를 소리 내어 읽었다",
  "예상과 실제 결과의 차이를 분석했다",
  "다음 검토에서 확인할 기준을 남겼다",
]);

const CONTEXTS = Object.freeze([
  "오전 검토가 끝난 뒤",
  "최종 배포를 시작하기 전에",
  "여러 사람이 함께 읽을 수 있도록",
  "개인 정보를 외부로 보내지 않고",
  "느린 장치에서도 같은 결과가 나오도록",
  "표현의 의미가 달라지지 않는 범위에서",
  "원문과 수정문을 나란히 살펴보며",
  "검증 절차와 한계를 분명히 밝히면서",
]);

function recordParts(index) {
  return {
    record: String(index + 1).padStart(6, "0"),
    genre: GENRES[index % GENRES.length],
    subject: SUBJECTS[(index * 5 + Math.floor(index / 7)) % SUBJECTS.length],
    action: ACTIONS[(index * 7 + Math.floor(index / 11)) % ACTIONS.length],
    context: CONTEXTS[(index * 3 + Math.floor(index / 13)) % CONTEXTS.length],
  };
}

function buildRecord(kind, index) {
  const { record, genre, subject, action, context } = recordParts(index);
  const sentence = `${context} ${subject} ${action}.`;

  if (kind === "plain") {
    return `[${genre} 기록 ${record}] ${sentence}\n검토 메모에는 근거와 담당자를 함께 적었다.\n`;
  }
  if (kind === "markdown") {
    return `### ${genre} 기록 ${record}\n\n- ${sentence}\n- 참고 코드: \`token_${record}\`은 검사 대상 문장이 아니다.\n\n`;
  }
  return `const record${record} = "${subject} ${action}"; // ${genre} 기록 ${record}: ${sentence}\n/* 검토 메모: 원문과 결과를 함께 보관했다. */\n`;
}

function buildText(kind, targetBytes) {
  const chunks = [];
  let byteLength = 0;
  let index = 0;

  while (true) {
    const record = buildRecord(kind, index);
    const recordBytes = Buffer.byteLength(record, "utf8");
    if (byteLength + recordBytes > targetBytes) {
      break;
    }
    chunks.push(record);
    byteLength += recordBytes;
    index += 1;
  }

  // ASCII padding makes every fixture an exact byte size without splitting a
  // multi-byte Korean character. It is deliberately inert for all source kinds.
  chunks.push(" ".repeat(targetBytes - byteLength));
  return chunks.join("");
}

export function buildBenchmarkFixtures() {
  return FIXTURE_KINDS.flatMap(({ id: kind, sourceKind }) =>
    Object.entries(BENCHMARK_SIZES).map(([size, targetBytes]) => {
      const text = buildText(kind, targetBytes);
      return {
        id: `${kind}-${size}`,
        sourceKind,
        size,
        byteLength: targetBytes,
        sha256: createHash("sha256").update(text, "utf8").digest("hex"),
        text,
      };
    }),
  );
}

export function writeBenchmarkFixtures(directory) {
  mkdirSync(directory, { recursive: true });
  const extensionBySourceKind = {
    plain_text: "txt",
    markdown: "md",
    typescript: "ts",
  };
  const fixtures = buildBenchmarkFixtures().map(({ text, ...fixture }) => {
    const path = `${fixture.id}.${extensionBySourceKind[fixture.sourceKind]}`;
    writeFileSync(join(directory, path), text, "utf8");
    return { ...fixture, path };
  });
  const manifest = { schemaVersion: 1, fixtures };
  writeFileSync(
    join(directory, "manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
    "utf8",
  );
  return manifest;
}
