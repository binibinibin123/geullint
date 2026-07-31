import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve as resolvePath } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { platformPackageFor, resolveBinary, run } from "../bin/geullint.js";

const packageDirectory = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");

function readPackage(name) {
  return JSON.parse(
    readFileSync(resolvePath(packageDirectory, "..", name, "package.json"), "utf8"),
  );
}

test("maps Windows x64 to its native package", () => {
  assert.deepEqual(platformPackageFor("win32", "x64"), {
    packageName: "geullint-win32-x64",
    executableName: "geullint.exe",
  });
});

test("maps Windows ARM64 to its native package", () => {
  assert.deepEqual(platformPackageFor("win32", "arm64"), {
    packageName: "geullint-win32-arm64",
    executableName: "geullint.exe",
  });
});

test("maps macOS Intel to its native package", () => {
  assert.deepEqual(platformPackageFor("darwin", "x64"), {
    packageName: "geullint-darwin-x64",
    executableName: "geullint",
  });
});

test("maps macOS Apple Silicon to its native package", () => {
  assert.deepEqual(platformPackageFor("darwin", "arm64"), {
    packageName: "geullint-darwin-arm64",
    executableName: "geullint",
  });
});

test("maps Linux x64 to its native package", () => {
  assert.deepEqual(platformPackageFor("linux", "x64"), {
    packageName: "geullint-linux-x64",
    executableName: "geullint",
  });
});

test("maps Linux ARM64 to its native package", () => {
  assert.deepEqual(platformPackageFor("linux", "arm64"), {
    packageName: "geullint-linux-arm64",
    executableName: "geullint",
  });
});

test("rejects unsupported platforms with an actionable error", () => {
  assert.throws(
    () => platformPackageFor("freebsd", "x64"),
    /Unsupported platform: freebsd-x64/,
  );
});

test("resolves the executable from the selected platform package", () => {
  const requests = [];
  const binary = resolveBinary("win32", "x64", (request) => {
    requests.push(request);
    return "C:/fixture/node_modules/geullint-win32-x64/bin/geullint.exe";
  });

  assert.equal(
    binary,
    "C:/fixture/node_modules/geullint-win32-x64/bin/geullint.exe",
  );
  assert.deepEqual(requests, ["geullint-win32-x64/bin/geullint.exe"]);
});

test("runs the selected binary with the caller arguments and returns its exit code", () => {
  const calls = [];
  const exitCode = run(["docs/"], {
    platform: "linux",
    arch: "x64",
    resolve: () => "/fixture/geullint",
    spawnSync: (binary, arguments_, options) => {
      calls.push({ binary, arguments_, options });
      return { status: 1 };
    },
  });

  assert.equal(exitCode, 1);
  assert.deepEqual(calls, [
    {
      binary: "/fixture/geullint",
      arguments_: ["docs/"],
      options: { stdio: "inherit" },
    },
  ]);
});

test("declares a Windows x64-only binary package", () => {
  const manifest = readPackage("geullint-win32-x64");

  assert.equal(manifest.name, "geullint-win32-x64");
  assert.deepEqual(manifest.os, ["win32"]);
  assert.deepEqual(manifest.cpu, ["x64"]);
  assert.deepEqual(manifest.files, ["bin", "NOTICE", "LICENSES"]);
});

test("declares native package manifests for every supported launcher target", () => {
  for (const [name, os, cpu] of [
    ["geullint-darwin-x64", "darwin", "x64"],
    ["geullint-darwin-arm64", "darwin", "arm64"],
    ["geullint-linux-x64", "linux", "x64"],
    ["geullint-win32-arm64", "win32", "arm64"],
    ["geullint-linux-arm64", "linux", "arm64"],
  ]) {
    const manifest = readPackage(name);

    assert.equal(manifest.name, name);
    assert.deepEqual(manifest.os, [os]);
    assert.deepEqual(manifest.cpu, [cpu]);
    assert.deepEqual(manifest.files, ["bin", "NOTICE", "LICENSES"]);
  }
});
