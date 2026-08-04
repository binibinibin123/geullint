import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve } from "node:path";

const workspace = resolve(import.meta.dirname, "..");
const wasm = resolve(workspace, "target", "wasm32-unknown-unknown", "release", "geullint_wasm.wasm");
const output = resolve(workspace, "apps", "playground", "pkg");
const sizeOptimizedRelease = {
  ...process.env,
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1",
  CARGO_PROFILE_RELEASE_LTO: "fat",
  CARGO_PROFILE_RELEASE_OPT_LEVEL: "z",
  CARGO_PROFILE_RELEASE_PANIC: "abort",
  CARGO_PROFILE_RELEASE_STRIP: "symbols",
};

execFileSync(
  "cargo",
  [
    "build",
    "-p",
    "geullint-wasm",
    "--features",
    "standard",
    "--locked",
    "--target",
    "wasm32-unknown-unknown",
    "--release",
  ],
  { cwd: workspace, env: sizeOptimizedRelease, stdio: "inherit" },
);
mkdirSync(output, { recursive: true });
execFileSync("wasm-bindgen", [wasm, "--target", "web", "--out-dir", output, "--no-typescript"], { cwd: workspace, stdio: "inherit" });
