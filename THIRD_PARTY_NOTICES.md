# Third-party notices

GeulLint's native CLI and language server embed Korean morphological-analysis
data through the dependencies below. This notice and the Apache-2.0 license
text in [`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt) must accompany
binary distributions.

## Lindera 4.0.1

- Project: <https://github.com/lindera/lindera>
- License: MIT
- Use: offline Korean tokenization runtime.

## Lindera ko-dic 4.0.1

- Project: <https://github.com/lindera/lindera>
- License: MIT
- Use: bundled `mecab-ko-dic` adapter and dictionary packaging.

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
