<p align="center"><img src="assets/brand/hero-light.svg" alt="GeulLint — ローカルで動く韓国語リンター" width="100%"></p>

<p align="center">
  <a href="README.md">한국어</a> · <a href="README.en.md">English</a> ·
  <a href="README.ja.md"><strong>日本語</strong></a> · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/binibinibin123/geullint/releases"><img alt="Release" src="https://img.shields.io/github/v/release/binibinibin123/geullint?display_name=tag&include_prereleases&sort=semver&color=ff5b35"></a>
  <a href="CHANGELOG.md"><img alt="Early alpha" src="https://img.shields.io/badge/status-early_alpha-dfff38?labelColor=18211c"></a>
  <a href="LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-f1efe6?labelColor=18211c"></a>
</p>

<p align="center"><strong>文章を外部へ送らない、オープンソースの韓国語文章校正・文法チェック</strong><br>韓国語の綴り・分かち書き・文法・文体をブラウザー、VS Code、ターミナルで確認します。文章は外部サーバーへ送信されません。</p>

<p align="center">
  <a href="https://binibinibin123.github.io/geullint/"><strong>今すぐ文章をチェック →</strong></a> ·
  <a href="#インストール">インストール</a> ·
  <a href="docs/rules.md">検証済みルールを見る</a>
</p>

<p align="center"><img src="assets/demo/geullint-demo.gif" alt="GeulLintブラウザーデモ" width="100%"></p>

## 書く場所を選ばない校正

ウェブで文章をすぐに確認し、作業が大きくなったら同じチェッカーをエディター・ターミナル・CIへ広げられます。

| 利用場面 | できること |
| --- | --- |
| VS Code | 入力中にリアルタイムで検査し、保守的な修正をワンクリックで適用 |
| CLI | 複数の文書やリポジトリーを1コマンドで一括検査 |
| CI | 文書品質の低下を自動で防ぎ、SARIF結果を生成 |
| 自分の語彙 | ユーザー辞書、dictionary overlay、プロジェクトrule packを追加 |

文章・診断・テレメトリーを外部サービスへ送信しません。公開済みのアルファ版は**v0.3.0-alpha.1**ですが、このリポジトリのルールカタログは固定の件数目標とは独立して更新されます。

## 特徴

GeulLintはブラウザー、エディター、ターミナルで完全に**オフライン**動作する韓国語文章校正・文法チェッカーです。同じ文章・入力種別・プロファイルなら、同じRustエンジンがどの環境でも同じルールIDと修正結果を返します。Markdown本文とJavaScript・TypeScript・Python・Rustのコメントを検査し、認識したコードと文字列の範囲は除外します。

| 項目 | 内容 |
| --- | --- |
| プライバシー | 文章・診断・テレメトリーのネットワーク送信0回 |
| 公開性 | 安定したルールID、説明、例、重要度、テスト |
| 自動化 | 人向け出力、JSON、SARIF 2.1.0、LSP、終了コード |
| カスタマイズ | ユーザー辞書、dictionary overlay、ローカルrule pack |
| 対応環境 | Windows・macOS・Linux、x64・ARM64 |

## インストール

インストール不要の[WebAssemblyプレイグラウンド](https://binibinibin123.github.io/geullint/)を利用できます。

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

スクリプトはGitHub ReleaseのSHA-256を検証します。実行前に[install.ps1](install.ps1)または[install.sh](install.sh)を確認できます。Rust環境がある場合：

```bash
cargo install --git https://github.com/binibinibin123/geullint --tag v0.3.0-alpha.1 --locked geullint-cli
```

手動アーカイブは[GitHub Releases](https://github.com/binibinibin123/geullint/releases)のfallbackです。

ディレクトリー検査では、既存の`.gitignore`とプロジェクト固有の`.geullintignore`パターンを適用します。

## ルールと品質

ルールカタログは、ルールメタデータと検証例から生成されます。ルール数は品質の主張ではありません。テストには誤用例と正常な反例が含まれますが、それだけで一般的なprecision・recallを示すことはできません。安全な自動修正は、確認が必要な提案より意図的に限定されます。評価の契約は[品質ゲート](docs/quality.md)と[コーパス評価の手順](docs/corpus-evaluation.md)をご覧ください。

[全ルール](docs/rules.md)、[品質ゲート](docs/quality.md)、[コーパス評価](docs/corpus-evaluation.md)、[オフライン方針](docs/offline.md)を参照してください。

## VS Code

<p align="center"><img src="assets/screenshots/vscode.png" alt="GeulLintの実装済みVS Code機能に基づくワークフロー概念図" width="100%"><br><sub>実装済み機能と実在するルールIDに基づく概念図です。配置はVS Codeのバージョンにより異なる場合があります。</sub></p>

[Releases](https://github.com/binibinibin123/geullint/releases)から環境に合うVSIXを取得し、`Extensions: Install from VSIX...`を選びます。ローカル言語サーバー同梱のため、Rust・Node.js・APIキーは不要です。

## 制限とコントリビューション

GeulLintは保守的なルールベースのリンターです。一般的な未知語辞書はまだ同梱していないため、公開ルール外の誤字を見逃すことがあります。長い文脈の意味判断や創作上の校閲を置き換えるものではありません。独立指標は、ライセンス・ハッシュ・レビュー記録を備えたコーパスでのみ公開します。

[CONTRIBUTING.md](CONTRIBUTING.md)、[ARCHITECTURE.md](ARCHITECTURE.md)、[ROADMAP.md](ROADMAP.md)をご覧ください。脆弱性は[SECURITY.md](SECURITY.md)の手順で報告してください。

MIT License。第三者ライセンスは[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)に記載しています。
