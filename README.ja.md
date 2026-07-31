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

現在のバージョンは**v0.2.0-alpha.1**で、100件のコアルールを収録しています。文章・診断・テレメトリーを外部サービスへ送信しません。

## 特徴

GeulLintはブラウザー、エディター、ターミナルで完全に**オフライン**動作する韓国語文章校正・文法チェッカーです。同じRustエンジンが、どこでも同じルールIDと修正結果を提供します。Markdown本文とJavaScript・TypeScript・Python・Rustのコメントを検査し、コードや文字列リテラルには触れません。

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
irm https://raw.githubusercontent.com/binibinibin123/geullint/master/install.ps1 | iex
geullint .
```

**macOS / Linux**

```bash
curl -fsSL https://raw.githubusercontent.com/binibinibin123/geullint/master/install.sh | sh
geullint .
```

スクリプトはGitHub ReleaseのSHA-256を検証します。実行前に[install.ps1](install.ps1)または[install.sh](install.sh)を確認できます。Rust環境がある場合：

```bash
cargo install --git https://github.com/binibinibin123/geullint --tag v0.2.0-alpha.1 --locked geullint-cli
```

手動アーカイブは[GitHub Releases](https://github.com/binibinibin123/geullint/releases)のfallbackです。

## ルールと品質

GeulLint v0.2.0-alpha.1は現在**100件の検証コアルール**を提供します。新たに確認した42件の語彙ルールには、84件の異なる誤用文と42件の正常な反例があります。また、KoLLA v2で全注釈者が正常と判定した249文では誤検出0件でした。小規模な正常文のみの監査であり、precision・recallの主張ではありません。詳細は[アルファ品質報告](docs/quality-report-v0.2.0-alpha.1.md)をご覧ください。

[全ルール](docs/rules.md)、[品質ゲート](docs/quality.md)、[コーパス評価](docs/corpus-evaluation.md)、[オフライン方針](docs/offline.md)を参照してください。

## VS Code

<p align="center"><img src="assets/screenshots/vscode.png" alt="VS CodeでのGeulLint診断、Quick Fix、ルール検索" width="100%"></p>

[Releases](https://github.com/binibinibin123/geullint/releases)から環境に合うVSIXを取得し、`Extensions: Install from VSIX...`を選びます。ローカル言語サーバー同梱のため、Rust・Node.js・APIキーは不要です。

## 制限とコントリビューション

GeulLintは保守的なルールベースのリンターです。長い文脈の意味判断や創作上の校閲を置き換えるものではありません。独立指標は、ライセンス・ハッシュ・レビュー記録を備えたコーパスでのみ公開します。

[CONTRIBUTING.md](CONTRIBUTING.md)、[ARCHITECTURE.md](ARCHITECTURE.md)、[ROADMAP.md](ROADMAP.md)をご覧ください。脆弱性は[SECURITY.md](SECURITY.md)の手順で報告してください。

MIT License。第三者ライセンスは[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)に記載しています。
