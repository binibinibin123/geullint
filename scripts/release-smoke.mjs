import { execFileSync, spawnSync } from "node:child_process";
import {
  chmodSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { basename, join, relative, resolve, sep } from "node:path";
import { tmpdir } from "node:os";
import { pathToFileURL } from "node:url";

export function parseChecksum(source, archiveName) {
  const match = source.trim().match(/^([a-f0-9]{64}) {2}(.+)$/u);
  if (!match) {
    throw new Error("release checksum must contain a lowercase SHA-256 digest");
  }
  if (match[2] !== archiveName) {
    throw new Error(`checksum archive name ${JSON.stringify(match[2])} does not match`);
  }
  return match[1];
}

export function validateArchiveEntries(entries, binaryName) {
  const normalized = entries.map((entry) => entry.split(sep).join("/"));
  const roots = new Set(normalized.map((entry) => entry.split("/")[0]));
  if (
    roots.size !== 1 ||
    !/^geullint-v.+-(?:win32|darwin|linux)-(?:x64|arm64)$/u.test([...roots][0])
  ) {
    throw new Error("release archive must contain one versioned root directory");
  }

  const root = [...roots][0];
  const executable = `${root}/${binaryName}`;
  const executables = normalized.filter((entry) => entry.split("/").at(-1) === binaryName);
  if (executables.length !== 1 || executables[0] !== executable) {
    throw new Error(`release archive must contain exactly one executable named ${binaryName}`);
  }
  for (const required of ["LICENSE", "NOTICE", "LICENSES/Apache-2.0.txt"]) {
    if (!normalized.includes(`${root}/${required}`)) {
      throw new Error(`release archive is missing ${required}`);
    }
  }
  return executable;
}

export function validateReleaseCatalog(catalog) {
  if (!Array.isArray(catalog?.rules) || catalog.ruleCount !== catalog.rules.length) {
    throw new Error("release catalogue ruleCount must match rules.length");
  }
  if (catalog.ruleCount === 0) {
    throw new Error("release catalogue must be nonempty");
  }
  if (catalog.ruleCount > 100) {
    throw new Error("release catalogue must contain at most 100 curated rules");
  }
}

function listFiles(root, current = root) {
  return readdirSync(current).flatMap((name) => {
    const path = join(current, name);
    if (statSync(path).isDirectory()) {
      return listFiles(root, path);
    }
    return [relative(root, path)];
  });
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function runChecked(binary, arguments_, label) {
  const result = spawnSync(binary, arguments_, {
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${label} failed with ${result.status}: ${result.stderr}`);
  }
  return result.stdout;
}

export function smokeReleaseArchive(archivePath, checksumPath, binaryName) {
  const archive = resolve(archivePath);
  const expectedDigest = parseChecksum(
    readFileSync(checksumPath, "utf8"),
    basename(archive),
  );
  if (sha256(archive) !== expectedDigest) {
    throw new Error("release archive SHA-256 does not match its checksum file");
  }

  const extractionRoot = mkdtempSync(join(tmpdir(), "geullint-release-smoke-"));
  try {
    if (!archive.endsWith(".zip") && !archive.endsWith(".tar.gz")) {
      throw new Error("release archive must be .zip or .tar.gz");
    }
    execFileSync("tar", ["-xf", archive, "-C", extractionRoot], {
      stdio: "inherit",
      windowsHide: true,
    });

    const binaryRelativePath = validateArchiveEntries(listFiles(extractionRoot), binaryName);
    const binary = join(extractionRoot, ...binaryRelativePath.split("/"));
    if (process.platform !== "win32") chmodSync(binary, 0o755);

    const version = runChecked(binary, ["--version"], "version smoke test");
    if (!/^geullint \d+\.\d+\.\d+/mu.test(version)) {
      throw new Error(`unexpected --version output: ${JSON.stringify(version)}`);
    }

    const catalog = JSON.parse(
      runChecked(binary, ["rules", "--format", "json"], "rule catalogue smoke test"),
    );
    validateReleaseCatalog(catalog);

    const fixture = join(extractionRoot, "smoke.txt");
    writeFileSync(fixture, "몇일 뒤에 만나요.", "utf8");
    const lint = spawnSync(binary, [fixture], {
      encoding: "utf8",
      windowsHide: true,
    });
    if (lint.error) throw lint.error;
    if (lint.status !== 1 || !lint.stdout.includes("spelling.lexical.myeochil")) {
      throw new Error("release binary did not diagnose the Korean smoke fixture");
    }
  } finally {
    rmSync(extractionRoot, { recursive: true, force: true });
  }
}

function main(arguments_) {
  if (arguments_.length !== 3) {
    throw new Error(
      "Usage: node scripts/release-smoke.mjs ARCHIVE CHECKSUM BINARY_NAME",
    );
  }
  smokeReleaseArchive(...arguments_);
  process.stdout.write(`verified release archive ${arguments_[0]}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 2;
  }
}
