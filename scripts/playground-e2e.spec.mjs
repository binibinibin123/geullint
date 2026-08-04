#!/usr/bin/env node

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { appendFileSync, createReadStream, existsSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const workspace = resolve(import.meta.dirname, "..");
const playground = resolve(workspace, "apps", "playground");
const packageDirectory = resolve(playground, "pkg");
const originHost = "127.0.0.1";
let currentStep = "startup";
const debugFile = process.env.GEULLINT_E2E_DEBUG_FILE;

function report(message) {
  console.log(message);
  if (debugFile) appendFileSync(debugFile, `${message}\n`);
}

const mimeTypes = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
  ".webmanifest": "application/manifest+json; charset=utf-8",
};

function ensurePackageBuilt() {
  if (existsSync(join(packageDirectory, "geullint_wasm.js"))) return;
  if (process.argv.includes("--no-build")) {
    throw new Error("apps/playground/pkg is missing; run node scripts/build-playground.mjs first");
  }
  execFileSync(process.execPath, ["scripts/build-playground.mjs"], {
    cwd: workspace,
    stdio: "inherit",
  });
}

function startServer() {
  const server = createServer((request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
      const relative = pathname === "/" ? "index.html" : pathname.slice(1);
      const candidate = resolve(playground, normalize(relative));
      const normalizedCandidate = candidate.replaceAll("\\", "/");
      const normalizedRoot = playground.replaceAll("\\", "/");
      if (normalizedCandidate !== normalizedRoot && !normalizedCandidate.startsWith(`${normalizedRoot}/`)) {
        response.writeHead(403).end("forbidden");
        return;
      }
      const file = statSync(candidate).isDirectory() ? join(candidate, "index.html") : candidate;
      response.writeHead(200, { "Content-Type": mimeTypes[extname(file)] ?? "application/octet-stream" });
      createReadStream(file).pipe(response);
    } catch {
      response.writeHead(404).end("not found");
    }
  });
  return new Promise((resolveServer, reject) => {
    server.once("error", reject);
    server.listen(0, originHost, () => {
      const address = server.address();
      resolveServer({ server, url: `http://${originHost}:${address.port}` });
    });
  });
}

async function waitForCorrection(page, expected) {
  await page.waitForFunction(
    (value) => document.querySelector("#corrected-output")?.value.includes(value),
    expected,
    { timeout: 20_000 },
  );
}

async function loadPlaywright() {
  const localModule = pathToFileURL(join(playground, "node_modules", "playwright", "index.mjs"));
  try {
    return await import(localModule.href);
  } catch (localError) {
    try {
      return await import("playwright");
    } catch (globalError) {
      const error = localError ?? globalError;
      if (process.env.GEULLINT_E2E_REQUIRED === "1") throw error;
      console.warn(`playground E2E skipped: ${error.message}`);
      return undefined;
    }
  }
}

async function runBrowser(browserType, viewportName, baseUrl, playwright) {
  const contextOptions = { permissions: ["clipboard-read", "clipboard-write"] };
  if (viewportName === "mobile") {
    const device = playwright.devices["Pixel 7"];
    if (!device) throw new Error("Playwright Pixel 7 mobile device descriptor is unavailable");
    Object.assign(contextOptions, device);
  }
  const label = `${browserType}/${viewportName}`;
  report(`[playground-e2e] launching ${label}`);
  const browser = await playwright[browserType].launch({ headless: true, timeout: 15_000 });
  const context = await browser.newContext(contextOptions);
  const externalRequests = [];
  const page = await context.newPage();
  currentStep = `${label}: page created`;
  page.on("console", (message) => report(`[browser:${label}] ${message.type()}: ${message.text()}`));
  page.on("pageerror", (error) => report(`[browser:${label}] pageerror: ${error.message}`));
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.origin !== baseUrl && url.protocol !== "data:") externalRequests.push(request.url());
  });
  try {
    currentStep = `${label}: goto`;
    report(`[playground-e2e] ${browserType}: loading ${baseUrl}`);
    await page.goto(`${baseUrl}/`, { waitUntil: "domcontentloaded" });
    currentStep = `${label}: service worker registration`;
    await page.waitForFunction(() => Boolean(navigator.serviceWorker), undefined, { timeout: 10_000 });
    await page.evaluate(() => navigator.serviceWorker.ready.then(() => true));
    currentStep = `${label}: first engine response`;
    report(`[playground-e2e] ${label}: service worker ready`);
    await page.locator("#scan").waitFor({ state: "visible" });
    report(`[playground-e2e] ${label}: waiting for first correction`);
    await waitForCorrection(page, "웬만");

    const editor = page.locator("#editor");
    const corrected = page.locator("#corrected-output");
    await editor.fill("몇일 뒤에 만나요.");
    await page.getByRole("button", { name: "문장 점검" }).click();
    await waitForCorrection(page, "며칠");
    assert.equal(await editor.inputValue(), "몇일 뒤에 만나요.");
    assert.equal(await corrected.inputValue(), "며칠 뒤에 만나요.");

    await page.locator("#copy-correction").click();
    await page.locator("#apply-correction").click();
    await page.waitForFunction(() => document.querySelector("#editor")?.value.includes("며칠"));
    await page.locator("#undo-correction").click();
    assert.equal(await editor.inputValue(), "몇일 뒤에 만나요.");
    await page.locator("#redo-correction").click();
    assert.equal(await editor.inputValue(), "며칠 뒤에 만나요.");

    await page.locator(".dictionary-control summary").click();
    await page.locator("#dictionary-entry").fill("내제품명");
    await page.locator("#dictionary-add").click();
    assert.match(await page.locator("#dictionary-list").innerText(), /내제품명/u);
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.locator(".dictionary-control summary").click();
    await page.locator("#dictionary-list").getByText("내제품명").waitFor({ state: "visible" });

    await context.setOffline(true);
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.locator("#scan").waitFor({ state: "visible" });
    await editor.fill("몇일 뒤에 만나요.");
    await page.getByRole("button", { name: "문장 점검" }).click();
    await waitForCorrection(page, "며칠");
    assert.deepEqual(externalRequests, []);

    return { browser: browserType, viewport: viewportName, externalRequests: externalRequests.length };
  } finally {
    await context.close().catch(() => {});
    await browser.close().catch(() => {});
  }
}

async function main() {
  const watchdog = setTimeout(() => {
    console.error(`playground E2E timed out at: ${currentStep}`);
    process.exit(1);
  }, Number(process.env.GEULLINT_E2E_TIMEOUT_MS ?? 180_000));
  const playwright = await loadPlaywright();
  if (!playwright) return;
  ensurePackageBuilt();
  const { server, url } = await startServer();
  try {
    const requested = process.env.GEULLINT_E2E_BROWSER ?? "chromium";
    const browsers = requested === "all" ? ["chromium", "firefox", "webkit"] : requested.split(",");
    const requestedViewports = process.env.GEULLINT_E2E_VIEWPORTS ?? "desktop";
    const viewports = requestedViewports === "all" ? ["desktop", "mobile"] : requestedViewports.split(",");
    const results = [];
    for (const browser of browsers) {
      if (!playwright[browser]) throw new Error(`unsupported browser: ${browser}`);
      for (const viewport of viewports) {
        if (!["desktop", "mobile"].includes(viewport)) throw new Error(`unsupported viewport: ${viewport}`);
        results.push(await runBrowser(browser, viewport, url, playwright));
      }
    }
    report(JSON.stringify({ browsers: results, offline: true, externalRequests: 0 }));
  } finally {
    server.close();
    clearTimeout(watchdog);
  }
}

main().catch((error) => {
  console.error(`playground E2E failed: ${error.stack ?? error}`);
  process.exitCode = 1;
});
