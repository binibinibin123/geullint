import assert from "node:assert/strict";
import { readFileSync, statSync } from "node:fs";
import test from "node:test";

const svgAssets = [
  "assets/brand/logo.svg",
  "assets/brand/mark.svg",
  "assets/brand/hero-light.svg",
  "assets/brand/hero-dark.svg",
];
const heroLight = readFileSync("assets/brand/hero-light.svg", "utf8");
const heroDark = readFileSync("assets/brand/hero-dark.svg", "utf8");
const socialPreview = readFileSync("assets/brand/social-preview.svg", "utf8");

function pngDimensions(path) {
  const bytes = readFileSync(path);
  assert.equal(bytes.subarray(1, 4).toString("ascii"), "PNG", `${path} is PNG`);
  return {
    width: bytes.readUInt32BE(16),
    height: bytes.readUInt32BE(20),
  };
}

test("ships accessible, compact SVG brand assets", () => {
  for (const path of svgAssets) {
    const source = readFileSync(path, "utf8");
    assert.match(source, /<title[^>]*>[^<]+<\/title>/u, `${path} has a title`);
    assert.match(source, /viewBox=/u, `${path} has a responsive viewBox`);
    assert.ok(statSync(path).size < 100_000, `${path} stays compact`);
  }
});

test("brand artwork leads with the private spelling-checker promise", () => {
  for (const source of [heroLight, heroDark, socialPreview]) {
    assert.match(source, /오픈소스 한국어 맞춤법 검사기/u);
    assert.match(source, /맞춤법·띄어쓰기·문법·문체/u);
  }
  assert.match(socialPreview, /브라우저 · VS Code · CLI · CI/u);
});

test("ships correctly sized social, extension, and screenshot PNGs", () => {
  assert.deepEqual(pngDimensions("assets/brand/social-preview.png"), {
    width: 1280,
    height: 640,
  });
  assert.deepEqual(pngDimensions("extensions/vscode-geullint/icon.png"), {
    width: 128,
    height: 128,
  });
  for (const path of [
    "assets/screenshots/playground.png",
    "assets/screenshots/vscode.png",
  ]) {
    const dimensions = pngDimensions(path);
    assert.ok(dimensions.width >= 1200, `${path} is wide enough`);
    assert.ok(dimensions.height >= 675, `${path} is tall enough`);
  }
});

test("ships a real demonstration GIF and references the extension icon", () => {
  const gif = readFileSync("assets/demo/geullint-demo.gif");
  assert.match(gif.subarray(0, 6).toString("ascii"), /^GIF8[79]a$/u);
  assert.ok(gif.length > 10_000, "demo animation contains rendered frames");

  const manifest = JSON.parse(
    readFileSync("extensions/vscode-geullint/package.json", "utf8"),
  );
  assert.equal(manifest.icon, "icon.png");
});
