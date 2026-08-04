# Third-party notices

## VS Code extension runtime bundle

The platform-specific VSIX bundles the following JavaScript packages into the
extension runtime. They provide the client side of the connection to the local
`geullint-lsp` process; no network service is used.

| Package | Version | License | Use in the VSIX |
| --- | --- | --- | --- |
| `vscode-languageclient` | `10.1.0` | MIT | VS Code language-client lifecycle and feature integration. |
| `vscode-jsonrpc` | `9.0.1` | MIT | JSON-RPC transport between the extension and the local language server. |
| `vscode-languageserver-protocol` | `3.18.2` | MIT | Language Server Protocol messages and capabilities. |
| `vscode-languageserver-types` | `3.18.0` | MIT | Shared Language Server Protocol data structures. |
| `vscode-languageserver-textdocument` | `1.0.13` | MIT | Text-document synchronization model used by the language client. |
| `semver` | `7.8.5` | ISC | Version-range handling used by the language client. |
| `minimatch` | `10.2.5` | BlueOak-1.0.0 | Glob matching used by language-client file watchers. |
| `balanced-match` | `4.0.4` | MIT | Balanced-brace parsing used by the glob matcher. |
| `brace-expansion` | `5.0.9` | MIT | Brace expansion used by the glob matcher. |

The license texts distributed with the VSIX are copied verbatim from the
installed packages:

- Microsoft VS Code language-server packages:
  [`MIT-Microsoft-vscode-languageserver-node.txt`](LICENSES/MIT-Microsoft-vscode-languageserver-node.txt)
- `semver`:
  [`ISC-semver.txt`](LICENSES/ISC-semver.txt)
- `minimatch`:
  [`BlueOak-minimatch.txt`](LICENSES/BlueOak-minimatch.txt)
- `balanced-match`:
  [`MIT-balanced-match.txt`](LICENSES/MIT-balanced-match.txt)
- `brace-expansion`:
  [`MIT-brace-expansion.txt`](LICENSES/MIT-brace-expansion.txt)

## Optional morphology dependencies

GeulLint's default release binaries do not enable the optional `morphology`
feature. Source builds that explicitly enable it embed Korean
morphological-analysis data through the dependencies below. This notice and
the Apache-2.0 license text in [`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt)
must accompany binary distributions that enable that feature.

## Lindera 4.0.1

- Project: <https://github.com/lindera/lindera>
- License: MIT
- Use: optional offline Korean tokenization runtime.

## Lindera ko-dic 4.0.1

- Project: <https://github.com/lindera/lindera>
- License: MIT
- Use: optional embedded `mecab-ko-dic` adapter and dictionary packaging.

## mecab-ko-dic 2.1.1-20180720

This software includes binary and/or source data from
`mecab-ko-dic-2.1.1-20180720`, originally distributed by the Eunjeon project:
<https://bitbucket.org/eunjeon/mecab-ko-dic/downloads/mecab-ko-dic-2.1.1-20180720.tar.gz>.

The data is licensed under the Apache License, Version 2.0. Its full text is
included in [`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt).

## RustCrypto SHA-2 0.10.9

- Project: <https://github.com/RustCrypto/hashes>
- License: MIT OR Apache-2.0
- Use: local SHA-256 verification for externally licensed corpus manifests.

The SHA-2 crate's MIT license text is included in
[`LICENSES/MIT-RustCrypto.txt`](LICENSES/MIT-RustCrypto.txt). The Apache-2.0
text is included in [`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt).

GeulLint does not currently bundle Korean Basic Dictionary data. If a future
release adds that separately licensed data, its exact source snapshot,
retrieval date, hash, and attribution will be recorded here before release.
