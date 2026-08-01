<p align="center"><img src="assets/brand/hero-light.svg" alt="GeulLint — Korean writing checks that stay local" width="100%"></p>

<p align="center">
  <a href="README.md">한국어</a> · <a href="README.en.md"><strong>English</strong></a> ·
  <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/binibinibin123/geullint/releases"><img alt="Release" src="https://img.shields.io/github/v/release/binibinibin123/geullint?display_name=tag&include_prereleases&sort=semver&color=ff5b35"></a>
  <a href="CHANGELOG.md"><img alt="Early alpha" src="https://img.shields.io/badge/status-early_alpha-dfff38?labelColor=18211c"></a>
  <a href="LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-f1efe6?labelColor=18211c"></a>
</p>

<p align="center"><strong>Open-source Korean spelling and grammar checker that keeps your writing local</strong><br>Check spelling, spacing, grammar, and style in your browser, VS Code, or terminal. Your writing is never sent to an external server.</p>

<p align="center">
  <a href="https://binibinibin123.github.io/geullint/"><strong>Check a sentence now →</strong></a> ·
  <a href="#install">Install</a> ·
  <a href="docs/rules.md">Browse the curated rules</a>
</p>

<p align="center"><img src="assets/demo/geullint-demo.gif" alt="GeulLint browser demo" width="100%"></p>

## One checker, wherever you write

Check a sentence on the web, then bring the same checker into your editor, terminal, and CI as your project grows.

| Where you write | What GeulLint does |
| --- | --- |
| VS Code | Checks as you type and applies conservative fixes with one click |
| CLI | Batch-checks documents and whole repositories with one command |
| CI | Prevents document-quality regressions and produces SARIF results |
| Your vocabulary | Adds a user dictionary, dictionary overlays, and project rule packs |

Writing, diagnostics, and telemetry are never sent to an external service. The published alpha release is **v0.3.0-alpha.1**; this repository's rule catalogue evolves independently of a fixed rule-count target.

## Why GeulLint

GeulLint is a Korean spelling and grammar checker that runs fully offline in the browser, your editor, and the terminal. For the same text, source kind, and profile, the same Rust engine produces the same rule IDs and corrections everywhere. It scans Markdown prose and comments in JavaScript, TypeScript, Python, and Rust while excluding code and string ranges it recognizes.

| Capability | Included |
| --- | --- |
| Privacy | 0 network requests for text, diagnostics, or telemetry |
| Open rules | Stable IDs, explanations, examples, severity, and tests |
| Automation | Human output, JSON, SARIF 2.1.0, LSP, exit codes |
| Customization | User dictionary, dictionary overlay, local rule packs |
| Platforms | Windows, macOS, and Linux on x64 and ARM64 |

## Install

Try the [local WebAssembly playground](https://binibinibin123.github.io/geullint/) without installing anything.

**Windows**

```powershell
$env:GEULLINT_VERSION='0.3.0-alpha.1'
irm https://raw.githubusercontent.com/binibinibin123/geullint/v0.3.0-alpha.1/install.ps1 | iex
geullint .
```

**macOS / Linux**

```bash
curl -fsSL https://raw.githubusercontent.com/binibinibin123/geullint/v0.3.0-alpha.1/install.sh | GEULLINT_VERSION=0.3.0-alpha.1 sh
geullint .
```

The scripts verify the SHA-256 checksum of the matching GitHub Release. Read [install.ps1](install.ps1) or [install.sh](install.sh) before running it. With a Rust toolchain:

```bash
cargo install --git https://github.com/binibinibin123/geullint --tag v0.3.0-alpha.1 --locked geullint-cli
```

Manual archives on [GitHub Releases](https://github.com/binibinibin123/geullint/releases) are the fallback.

## Use

```bash
geullint .
geullint --format json docs/
geullint --format sarif docs/ > geullint.sarif
geullint rules --format markdown
geullint --dictionary-overlay .geullint.overlay docs/
geullint --rule-pack .geullint-rules.yaml docs/
geullint lsp --stdio
```

Directory scans honor existing `.gitignore` files and project-specific `.geullintignore` patterns.

The rule catalogue is generated from rule metadata and checked examples. A rule count is not a quality claim: tests include error cases and normal counterexamples, but those cases do not establish general precision or recall. Safe automatic corrections are deliberately narrower than review suggestions. See the [quality gates](docs/quality.md) and [corpus evaluation protocol](docs/corpus-evaluation.md) for the evaluation contract.

See the [rule catalogue](docs/rules.md), [quality gates](docs/quality.md), [corpus evaluation protocol](docs/corpus-evaluation.md), and [offline policy](docs/offline.md).

## VS Code

<p align="center"><img src="assets/screenshots/vscode.png" alt="Concept view based on GeulLint's actual VS Code diagnostics, quick fixes, and rule search" width="100%"><br><sub>Concept view based on implemented features and real rule IDs. Layout may vary by VS Code version.</sub></p>

Download the platform-matched VSIX from [Releases](https://github.com/binibinibin123/geullint/releases) and choose `Extensions: Install from VSIX...`. The local language server is bundled; no Rust, Node.js, API key, or network service is required.

## Limits and contributing

GeulLint is a conservative rule-based linter, not a substitute for semantic review or creative editorial judgment. It does not yet ship a general OOV dictionary, so typos outside the published rules can be missed. Low-confidence suggestions are informational or intentionally left without an automatic fix. Independent metrics are published only with licensed, hashed, reviewable corpora.

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), the [architecture](ARCHITECTURE.md), and the [roadmap](ROADMAP.md). Report vulnerabilities through [SECURITY.md](SECURITY.md).

MIT licensed. Third-party attributions are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
