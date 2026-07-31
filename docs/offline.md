# Offline and privacy

GeulLint is designed so that checking a document is a local operation:

- `geullint-core` performs rule matching in process.
- `geullint` reads only paths passed on the local machine.
- `geullint-lsp` communicates only over its local standard-input/output transport.
- The VS Code client launches that local LSP process and does not configure telemetry.

The project does not include an HTTP client, API key, account, cloud service, or in-product analytics. Package-manager downloads while building the project and editor/marketplace update checks are outside linting itself; use locked dependencies and verified release artifacts when your environment requires supply-chain controls.

If this boundary changes, it is a release-blocking change and must be documented prominently in the README and changelog.
