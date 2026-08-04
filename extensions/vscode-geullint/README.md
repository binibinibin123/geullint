# GeulLint for VS Code

GeulLint checks Korean prose entirely on your machine. It supports Markdown and text, plus comments in JavaScript, TypeScript, Python, and Rust.

The released VSIX contains the platform-matching language server. For a local checkout, build the Rust server and either put `geullint-lsp` on `PATH` or set `geullint.serverPath` to its absolute path.

```bash
cargo build --release -p geullint-lsp --features standard
```

The extension creates LSP diagnostics and offers Quick Fixes only when GeulLint marks a correction safe. It has no account, API key, or telemetry.

## Settings

- `geullint.profile`: choose `default`, `strict`, or `editorial` rules for each workspace.
- `geullint.engine`: choose `standard` (bundled candidate suggestions), `compact` (smallest conservative engine), or `context` (experimental local ranker). Standard and context candidates are Review-only; Quick Fix remains Safe-only.
- `geullint.userDictionary`: project-specific names and terms to accept.
- `geullint.dictionaryOverlay`: additional project terms accepted by dictionary-aware lexical rules.
- `geullint.rulePacks`: local YAML rule packs. Relative paths are resolved from the first workspace folder.

Changing these settings immediately rechecks open files. Safe replacements from rule packs also appear as Quick Fixes. Tagged releases attach a platform-matched VSIX that already contains the offline language server; use VS Code's **Install from VSIX…** command to install it without Rust or npm.
