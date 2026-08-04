import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const seedPath = resolve("dictionaries/standard-ko-v1.seed.tsv");
const corpusPaths = [
  resolve("corpus/curated-alpha-v1.jsonl"),
  resolve("corpus/safety-regressions-v1.jsonl"),
  resolve("corpus/seed-v1.jsonl"),
];

function parseSeed(source) {
  const rows = new Map();
  for (const line of source.split(/\r?\n/u).slice(1)) {
    if (!line.trim()) continue;
    const [surface, pos, frequency] = line.split("\t");
    rows.set(surface, { surface, pos, frequency: Number(frequency) });
  }
  return rows;
}

async function main() {
  const rows = parseSeed(await readFile(seedPath, "utf8"));
  for (const corpusPath of corpusPaths) {
    const source = await readFile(corpusPath, "utf8");
    for (const line of source.split(/\r?\n/u)) {
      if (!line.trim()) continue;
      const item = JSON.parse(line);
      for (const surface of String(item.text).match(/[가-힣]{2,}/gu) ?? []) {
        const previous = rows.get(surface);
        if (previous) {
          previous.frequency += 10;
        } else {
          rows.set(surface, { surface, pos: "UNK", frequency: 10 });
        }
      }
    }
  }
  const output = ["surface\tpos\tfrequency"];
  for (const row of [...rows.values()].sort((left, right) => left.surface.localeCompare(right.surface, "ko"))) {
    output.push(`${row.surface}\t${row.pos}\t${row.frequency}`);
  }
  await writeFile(seedPath, `${output.join("\n")}\n`);
  console.log(JSON.stringify({ entries: rows.size, seedPath }, null, 2));
}

main().catch((error) => {
  console.error(`expand lexicon: ${error.message}`);
  process.exitCode = 2;
});
