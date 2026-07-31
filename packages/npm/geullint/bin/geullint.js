#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import process from "node:process";
import { fileURLToPath } from "node:url";

const requireFromHere = createRequire(import.meta.url);

const PLATFORM_PACKAGES = new Map([
  ["win32-x64", { packageName: "geullint-win32-x64", executableName: "geullint.exe" }],
  ["win32-arm64", { packageName: "geullint-win32-arm64", executableName: "geullint.exe" }],
  ["darwin-x64", { packageName: "geullint-darwin-x64", executableName: "geullint" }],
  ["darwin-arm64", { packageName: "geullint-darwin-arm64", executableName: "geullint" }],
  ["linux-x64", { packageName: "geullint-linux-x64", executableName: "geullint" }],
  ["linux-arm64", { packageName: "geullint-linux-arm64", executableName: "geullint" }],
]);

export function platformPackageFor(platform, arch) {
  const target = `${platform}-${arch}`;
  const packageDetails = PLATFORM_PACKAGES.get(target);

  if (!packageDetails) {
    throw new Error(
      `Unsupported platform: ${target}. GeulLint supports Windows x64/arm64, macOS x64/arm64, and Linux x64/arm64.`,
    );
  }

  return packageDetails;
}

export function resolveBinary(platform, arch, resolve = requireFromHere.resolve) {
  const { packageName, executableName } = platformPackageFor(platform, arch);
  return resolve(`${packageName}/bin/${executableName}`);
}

export function run(arguments_, dependencies = {}) {
  const {
    platform = process.platform,
    arch = process.arch,
    resolve = requireFromHere.resolve,
    spawnSync: spawn = spawnSync,
  } = dependencies;

  let binary;
  try {
    binary = resolveBinary(platform, arch, resolve);
  } catch (error) {
    process.stderr.write(`geullint: ${error.message}\n`);
    return 2;
  }

  const result = spawn(binary, arguments_, { stdio: "inherit" });
  if (result.error) {
    process.stderr.write(`geullint: unable to start bundled binary: ${result.error.message}\n`);
    return 2;
  }

  return result.status ?? 2;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exitCode = run(process.argv.slice(2));
}
