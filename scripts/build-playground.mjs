import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve } from "node:path";

const workspace = resolve(import.meta.dirname, "..");
const wasm = resolve(workspace, "target", "wasm32-unknown-unknown", "release", "geullint_wasm.wasm");
const output = resolve(workspace, "apps", "playground", "pkg");

execFileSync("cargo", ["build", "-p", "geullint-wasm", "--target", "wasm32-unknown-unknown", "--release"], { cwd: workspace, stdio: "inherit" });
mkdirSync(output, { recursive: true });
execFileSync("wasm-bindgen", [wasm, "--target", "web", "--out-dir", output, "--no-typescript"], { cwd: workspace, stdio: "inherit" });
