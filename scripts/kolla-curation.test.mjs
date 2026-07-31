import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  symlink,
  unlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const moduleUrl = new URL("./curate-kolla-v2-gold.mjs", import.meta.url);
const curation = existsSync(fileURLToPath(moduleUrl)) ? await import(moduleUrl) : {};
const repositoryRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));

test("converts explicitly reviewed KoLLA candidates into exact GeulLint gold cases", () => {
  assert.equal(typeof curation.curateGoldCases, "function");

  const goldCases = curation.curateGoldCases(
    [
      {
        id: "kolla-v2-review-1",
        text: "몇일 뒤에 만나요.",
        sourceTokens: ["몇일", "뒤에", "만나요", "."],
        references: [],
      },
    ],
    {
      schemaVersion: 1,
      cases: [
        {
          reviewId: "kolla-v2-review-1",
          expectedDiagnostics: [
            {
              ruleId: "spelling.lexical.myeochil",
              range: { start: 0, end: 6 },
              suggestions: ["며칠"],
            },
          ],
        },
      ],
    },
  );

  assert.deepEqual(goldCases, [
    {
      id: "kolla-v2-review-1",
      text: "몇일 뒤에 만나요.",
      sourceKind: "plain_text",
      expectedDiagnostics: [
        {
          ruleId: "spelling.lexical.myeochil",
          range: { start: 0, end: 6 },
          suggestions: ["며칠"],
        },
      ],
    },
  ]);
});

test("rejects a reviewed mapping without at least one exact suggestion", () => {
  assert.throws(
    () =>
      curation.curateGoldCases(
        [{ id: "kolla-v2-review-1", text: "며칠 뒤에 만나요.", references: [] }],
        {
          schemaVersion: 1,
          cases: [
            {
              reviewId: "kolla-v2-review-1",
              expectedDiagnostics: [
                {
                  ruleId: "spelling.lexical.myeochil",
                  range: { start: 0, end: 6 },
                  suggestions: [],
                },
              ],
            },
          ],
        },
      ),
    /invalid exact diagnostic/u,
  );
});

test("requires string, nonblank review IDs in queues and mappings", () => {
  for (const invalidId of [17, "   "]) {
    assert.throws(
      () => curateSingle({ reviewId: invalidId, selectionReviewId: invalidId }),
      /invalid review ID/u,
    );
    assert.throws(
      () => curateSingle({ selectionReviewId: invalidId }),
      /invalid review ID/u,
    );
  }
});

test("requires string, nonblank rule IDs and suggestions", () => {
  for (const invalidRuleId of [17, "   "]) {
    assert.throws(
      () => curateSingle({ ruleId: invalidRuleId }),
      /invalid exact diagnostic/u,
    );
  }
  for (const invalidSuggestion of [17, "   "]) {
    assert.throws(
      () => curateSingle({ suggestions: [invalidSuggestion] }),
      /invalid exact diagnostic/u,
    );
  }
});

test("requires review text to be a string with nonblank content", () => {
  for (const invalidText of [17, "   "]) {
    const review = validReviewCase();
    review.text = invalidText;
    assert.throws(
      () => curation.curateGoldCases([review], validMapping()),
      /invalid review text/u,
    );
  }
});

test("rejects UTF-8 mid-byte ranges and unknown or duplicate review IDs", () => {
  assert.throws(
    () => curateSingle({ range: { start: 1, end: 6 } }),
    /invalid exact diagnostic/u,
  );
  assert.throws(
    () => curateSingle({ selectionReviewId: "kolla-v2-review-unknown" }),
    /unknown or duplicate review ID/u,
  );
  assert.throws(
    () =>
      curation.curateGoldCases(
        [
          validReviewCase(),
          validReviewCase(),
        ],
        validMapping(),
      ),
    /invalid or duplicate case ID/u,
  );
  const duplicateSelection = validMapping();
  duplicateSelection.cases.push(structuredClone(duplicateSelection.cases[0]));
  assert.throws(
    () => curation.curateGoldCases([validReviewCase()], duplicateSelection),
    /unknown or duplicate review ID/u,
  );
});

test("requires two independent reviewers and a separate adjudicator for release-quality gold", () => {
  const accepted = validMapping();
  accepted.cases[0].independentReviews = [
    validIndependentReview("reviewer-a"),
    validIndependentReview("reviewer-b"),
  ];
  accepted.cases[0].adjudicatedBy = "adjudicator-c";
  assert.equal(
    curation.curateGoldCases([validReviewCase()], accepted, {
      requireIndependentReview: true,
    }).length,
    1,
  );

  const scenarios = [
    {
      name: "one reviewer",
      mutate: (mapping) => {
        mapping.cases[0].independentReviews = [validIndependentReview("reviewer-a")];
        mapping.cases[0].adjudicatedBy = "adjudicator-c";
      },
      error: /at least two independent reviewers/u,
    },
    {
      name: "duplicate reviewer",
      mutate: (mapping) => {
        mapping.cases[0].independentReviews = [
          validIndependentReview("reviewer-a"),
          validIndependentReview("reviewer-a"),
        ];
        mapping.cases[0].adjudicatedBy = "adjudicator-c";
      },
      error: /unique independent reviewer/u,
    },
    {
      name: "malformed reviewer diagnostic",
      mutate: (mapping) => {
        mapping.cases[0].independentReviews = [
          validIndependentReview("reviewer-a", { suggestions: [] }),
          validIndependentReview("reviewer-b"),
        ];
        mapping.cases[0].adjudicatedBy = "adjudicator-c";
      },
      error: /invalid exact diagnostic/u,
    },
    {
      name: "reviewer adjudicator conflict",
      mutate: (mapping) => {
        mapping.cases[0].independentReviews = [
          validIndependentReview("reviewer-a"),
          validIndependentReview("reviewer-b"),
        ];
        mapping.cases[0].adjudicatedBy = "reviewer-a";
      },
      error: /separate adjudicator/u,
    },
  ];
  for (const scenario of scenarios) {
    const mapping = validMapping();
    scenario.mutate(mapping);
    assert.throws(
      () =>
        curation.curateGoldCases([validReviewCase()], mapping, {
          requireIndependentReview: true,
        }),
      scenario.error,
      scenario.name,
    );
  }
});

test("CLI release-quality mode refuses curation without independent review metadata", async () => {
  const directory = await mkdtemp(join(tmpdir(), "geullint-kolla-independent-review-"));
  try {
    const fixture = await writeCurationInputs(directory);
    const rejected = runCuration(fixture, ["--require-independent-review"]);
    assert.equal(rejected.status, 2);
    assert.match(rejected.stderr, /at least two independent reviewers/u);

    const mapping = validMapping();
    mapping.cases[0].independentReviews = [
      validIndependentReview("reviewer-a"),
      validIndependentReview("reviewer-b"),
    ];
    mapping.cases[0].adjudicatedBy = "adjudicator-c";
    await writeFile(fixture.mapping, JSON.stringify(mapping));

    const accepted = runCuration(fixture, ["--require-independent-review"]);
    assert.equal(accepted.status, 0, accepted.stderr);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("verification enforces whether a curated bundle required independent review", async () => {
  const ordinaryDirectory = await mkdtemp(join(tmpdir(), "geullint-kolla-ordinary-review-"));
  const releaseDirectory = await mkdtemp(join(tmpdir(), "geullint-kolla-release-review-"));
  try {
    const ordinary = await createCuratedFixture(ordinaryDirectory);
    const ordinaryVerification = runCurationVerification(ordinary, [
      "--require-independent-review",
    ]);
    assert.equal(ordinaryVerification.status, 2);
    assert.match(ordinaryVerification.stderr, /does not require independent review/u);

    const release = await writeCurationInputs(releaseDirectory);
    const mapping = validMapping();
    mapping.cases[0].independentReviews = [
      validIndependentReview("reviewer-a"),
      validIndependentReview("reviewer-b"),
    ];
    mapping.cases[0].adjudicatedBy = "adjudicator-c";
    await writeFile(release.mapping, JSON.stringify(mapping));
    const curated = runCuration(release, ["--require-independent-review"]);
    assert.equal(curated.status, 0, curated.stderr);

    const releaseVerification = runCurationVerification(release, [
      "--require-independent-review",
    ]);
    assert.equal(releaseVerification.status, 0, releaseVerification.stderr);
  } finally {
    await rm(ordinaryDirectory, { recursive: true, force: true });
    await rm(releaseDirectory, { recursive: true, force: true });
  }
});

test("writes and verifies input hashes plus a provenance sidecar", async () => {
  const directory = await mkdtemp(join(tmpdir(), "geullint-kolla-curation-"));
  try {
    const fixture = await createCuratedFixture(directory);
    const corpusBytes = await readFile(fixture.corpus);
    const manifestBytes = await readFile(fixture.manifest);
    const provenanceBytes = await readFile(fixture.provenance);
    const provenance = JSON.parse(provenanceBytes.toString("utf8"));
    const sidecar = await readFile(fixture.provenanceSha256, "utf8");

    assert.equal(provenance.reviewQueueSha256, digest(await readFile(fixture.reviewQueue)));
    assert.equal(provenance.mappingSha256, digest(await readFile(fixture.mapping)));
    assert.equal(provenance.corpusSha256, digest(corpusBytes));
    assert.equal(provenance.manifestSha256, digest(manifestBytes));
    assert.equal(
      sidecar,
      `${digest(provenanceBytes)}  kolla-v2-curated-gold.provenance.json\n`,
    );

    const verified = runCurationVerification(fixture);
    assert.equal(verified.status, 0, verified.stderr);

    await writeFile(fixture.provenance, `${provenanceBytes.toString("utf8")} `);
    const tampered = runCurationVerification(fixture);
    assert.equal(tampered.status, 2);
    assert.match(tampered.stderr, /provenance SHA-256 does not match/u);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("verification rejects manifest byte tampering through the provenance chain", async () => {
  const directory = await mkdtemp(join(tmpdir(), "geullint-kolla-manifest-chain-"));
  try {
    const fixture = await createCuratedFixture(directory);
    await writeFile(fixture.manifest, `${await readFile(fixture.manifest, "utf8")} `);

    const tampered = runCurationVerification(fixture);
    assert.equal(tampered.status, 2);
    assert.match(tampered.stderr, /manifest SHA-256 does not match/u);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("verification validates manifest schema, corpus path, and corpus SHA-256", async () => {
  const scenarios = [
    {
      name: "schema",
      mutate: (manifest) => {
        manifest.schemaVersion = 2;
      },
      error: /manifest schemaVersion must be 1/u,
    },
    {
      name: "path",
      mutate: (manifest) => {
        manifest.corpusPath = "different-corpus.jsonl";
      },
      error: /manifest corpusPath does not match/u,
    },
    {
      name: "corpus-sha",
      mutate: (manifest) => {
        manifest.sha256 = "0".repeat(64);
      },
      error: /manifest corpus SHA-256 does not match/u,
    },
    {
      name: "name",
      mutate: (manifest) => {
        manifest.name = "Tampered KoLLA corpus";
      },
      error: /manifest name does not match/u,
    },
    {
      name: "license",
      mutate: (manifest) => {
        manifest.license = "MIT";
      },
      error: /manifest license does not match/u,
    },
    {
      name: "source-url",
      mutate: (manifest) => {
        manifest.sourceUrl = "https://example.invalid/tampered";
      },
      error: /manifest sourceUrl does not match/u,
    },
  ];
  for (const scenario of scenarios) {
    const directory = await mkdtemp(join(tmpdir(), `geullint-kolla-${scenario.name}-`));
    try {
      const fixture = await createCuratedFixture(directory);
      await rewriteTrustedManifest(fixture, scenario.mutate);

      const tampered = runCurationVerification(fixture);
      assert.equal(tampered.status, 2);
      assert.match(tampered.stderr, scenario.error);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  }
});

test("publishes atomically without overwriting an existing output directory", async () => {
  const directory = await mkdtemp(join(tmpdir(), "geullint-kolla-atomic-"));
  try {
    const fixture = await writeCurationInputs(directory);
    await mkdir(fixture.output);
    const sentinel = join(fixture.output, "sentinel.txt");
    await writeFile(sentinel, "preserve me");

    const blocked = runCuration(fixture);
    assert.equal(blocked.status, 2);
    assert.equal(await readFile(sentinel, "utf8"), "preserve me");
    assert.deepEqual(await readdir(fixture.output), ["sentinel.txt"]);
    assert.equal(
      (await readdir(directory)).some((entry) => entry.startsWith("output.tmp-")),
      false,
    );

    await rm(fixture.output, { recursive: true });
    const retried = runCuration(fixture);
    assert.equal(retried.status, 0, retried.stderr);
    assert.deepEqual((await readdir(fixture.output)).sort(), [
      "kolla-v2-curated-gold.jsonl",
      "kolla-v2-curated-gold.manifest.json",
      "kolla-v2-curated-gold.provenance.json",
      "kolla-v2-curated-gold.provenance.sha256",
    ]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("CLI evaluates curated exact diagnostics and rejects corpus hash tampering", async () => {
  const directory = await mkdtemp(join(tmpdir(), "geullint-kolla-evaluator-"));
  try {
    const fixture = await createCuratedFixture(directory);
    const cli = buildGeullintCli();
    assert.equal(cli.status, 0, cli.stderr);

    const evaluated = spawnSync(cli.binaryPath, ["--corpus-manifest", fixture.manifest], {
      cwd: repositoryRoot,
      encoding: "utf8",
    });
    assert.equal(evaluated.status, 0, evaluated.stderr);
    const report = JSON.parse(evaluated.stdout);
    assert.equal(report.truePositives, 1);
    assert.equal(report.falsePositives, 0);
    assert.equal(report.falseNegatives, 0);

    await writeFile(fixture.corpus, `${await readFile(fixture.corpus, "utf8")}\n`);
    const tampered = spawnSync(
      cli.binaryPath,
      ["--corpus-manifest", fixture.manifest],
      {
        cwd: repositoryRoot,
        encoding: "utf8",
      },
    );
    assert.equal(tampered.status, 2);
    assert.match(tampered.stderr, /corpus SHA-256 does not match/u);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("CLI integration resolves a CARGO_TARGET_DIR override from cargo metadata", async () => {
  const directory = await mkdtemp(join(tmpdir(), "geullint-kolla-cargo-target-"));
  try {
    const fixture = await createCuratedFixture(directory);
    const defaultCli = buildGeullintCli();
    assert.equal(defaultCli.status, 0, defaultCli.stderr);
    const cargoTargetDirectory = join(directory, "cargo-target");
    await symlink(
      defaultCli.targetDirectory,
      cargoTargetDirectory,
      process.platform === "win32" ? "junction" : "dir",
    );
    try {
      const cli = buildGeullintCli({
        ...process.env,
        CARGO_TARGET_DIR: cargoTargetDirectory,
      });
      assert.equal(cli.status, 0, cli.stderr);
      assert.equal(resolve(cli.targetDirectory), resolve(cargoTargetDirectory));

      const evaluated = spawnSync(cli.binaryPath, ["--corpus-manifest", fixture.manifest], {
        cwd: repositoryRoot,
        encoding: "utf8",
      });
      assert.equal(evaluated.status, 0, evaluated.stderr);
      const report = JSON.parse(evaluated.stdout);
      assert.equal(report.truePositives, 1);
      assert.equal(report.falsePositives, 0);
      assert.equal(report.falseNegatives, 0);
    } finally {
      await unlink(cargoTargetDirectory);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

function curateSingle({
  reviewId = "kolla-v2-review-1",
  selectionReviewId = "kolla-v2-review-1",
  ruleId = "spelling.lexical.myeochil",
  range = { start: 0, end: 6 },
  suggestions = ["며칠"],
} = {}) {
  return curation.curateGoldCases(
    [validReviewCase(reviewId)],
    validMapping({ selectionReviewId, ruleId, range, suggestions }),
  );
}

function validReviewCase(id = "kolla-v2-review-1") {
  return {
    id,
    text: "몇일 뒤에 만나요.",
    sourceTokens: ["몇일", "뒤에", "만나요", "."],
    references: [],
  };
}

function validMapping({
  selectionReviewId = "kolla-v2-review-1",
  ruleId = "spelling.lexical.myeochil",
  range = { start: 0, end: 6 },
  suggestions = ["며칠"],
} = {}) {
  return {
    schemaVersion: 1,
    cases: [
      {
        reviewId: selectionReviewId,
        expectedDiagnostics: [{ ruleId, range, suggestions }],
      },
    ],
  };
}

function validIndependentReview(reviewer, diagnostic = {}) {
  return {
    reviewer,
    expectedDiagnostics: [
      {
        ruleId: "spelling.lexical.myeochil",
        range: { start: 0, end: 6 },
        suggestions: ["며칠"],
        ...diagnostic,
      },
    ],
  };
}

async function createCuratedFixture(directory) {
  const fixture = await writeCurationInputs(directory);
  const result = runCuration(fixture);
  assert.equal(result.status, 0, result.stderr);
  return {
    ...fixture,
    corpus: join(fixture.output, "kolla-v2-curated-gold.jsonl"),
    manifest: join(fixture.output, "kolla-v2-curated-gold.manifest.json"),
    provenance: join(fixture.output, "kolla-v2-curated-gold.provenance.json"),
    provenanceSha256: join(fixture.output, "kolla-v2-curated-gold.provenance.sha256"),
  };
}

async function writeCurationInputs(directory) {
  const reviewQueue = join(directory, "review-queue.jsonl");
  const mapping = join(directory, "mapping.json");
  const output = join(directory, "output");
  await writeFile(
    reviewQueue,
    `${JSON.stringify({
      id: "kolla-v2-review-1",
      text: "몇일 뒤에 만나요.",
      sourceTokens: ["몇일", "뒤에", "만나요", "."],
      references: [],
    })}\n`,
  );
  await writeFile(
    mapping,
    JSON.stringify({
      schemaVersion: 1,
      cases: [
        {
          reviewId: "kolla-v2-review-1",
          expectedDiagnostics: [
            {
              ruleId: "spelling.lexical.myeochil",
              range: { start: 0, end: 6 },
              suggestions: ["며칠"],
            },
          ],
        },
      ],
    }),
  );
  return { reviewQueue, mapping, output };
}

function runCuration(fixture, extraArguments = []) {
  return spawnSync(
    process.execPath,
    [
      "scripts/curate-kolla-v2-gold.mjs",
      "--review-queue",
      fixture.reviewQueue,
      "--mapping",
      fixture.mapping,
      "--out-dir",
      fixture.output,
      ...extraArguments,
    ],
    { cwd: repositoryRoot, encoding: "utf8" },
  );
}

async function rewriteTrustedManifest(fixture, mutate) {
  const manifest = JSON.parse(await readFile(fixture.manifest, "utf8"));
  mutate(manifest);
  const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
  await writeFile(fixture.manifest, manifestBytes);

  const provenance = JSON.parse(await readFile(fixture.provenance, "utf8"));
  provenance.manifestSha256 = digest(manifestBytes);
  const provenanceBytes = Buffer.from(`${JSON.stringify(provenance, null, 2)}\n`);
  await writeFile(fixture.provenance, provenanceBytes);
  await writeFile(
    fixture.provenanceSha256,
    `${digest(provenanceBytes)}  kolla-v2-curated-gold.provenance.json\n`,
  );
}

function runCurationVerification(fixture, extraArguments = []) {
  return spawnSync(
    process.execPath,
    [
      "scripts/curate-kolla-v2-gold.mjs",
      "--verify",
      "--review-queue",
      fixture.reviewQueue,
      "--mapping",
      fixture.mapping,
      "--out-dir",
      fixture.output,
      ...extraArguments,
    ],
    { cwd: repositoryRoot, encoding: "utf8" },
  );
}

function buildGeullintCli(environment = process.env) {
  const options = {
    cwd: repositoryRoot,
    encoding: "utf8",
    env: environment,
  };
  const build = spawnSync("cargo", ["build", "--quiet", "-p", "geullint-cli"], options);
  if (build.status !== 0) {
    return build;
  }
  const metadata = spawnSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1"],
    options,
  );
  if (metadata.status !== 0) {
    return metadata;
  }
  const targetDirectory = JSON.parse(metadata.stdout).target_directory;
  return {
    ...build,
    targetDirectory,
    binaryPath: join(
      targetDirectory,
      "debug",
      process.platform === "win32" ? "geullint.exe" : "geullint",
    ),
  };
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
