<p align="center"><img src="assets/brand/hero-light.svg" alt="GeulLint — 完全本地运行的韩语检查器" width="100%"></p>

<p align="center">
  <a href="README.md">한국어</a> · <a href="README.en.md">English</a> ·
  <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md"><strong>简体中文</strong></a>
</p>

<p align="center">
  <a href="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/binibinibin123/geullint/releases"><img alt="Release" src="https://img.shields.io/github/v/release/binibinibin123/geullint?display_name=tag&include_prereleases&sort=semver&color=ff5b35"></a>
  <a href="CHANGELOG.md"><img alt="Early alpha" src="https://img.shields.io/badge/status-early_alpha-dfff38?labelColor=18211c"></a>
  <a href="LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-f1efe6?labelColor=18211c"></a>
</p>

<p align="center"><strong>不上传文本的开源韩语拼写与语法检查器</strong><br>在浏览器、VS Code 和终端中检查韩语拼写、空格、语法与文体。文本不会发送到外部服务器。</p>

<p align="center">
  <a href="https://binibinibin123.github.io/geullint/"><strong>立即检查句子 →</strong></a> ·
  <a href="#安装">安装</a> ·
  <a href="docs/rules.md">查看已验证规则</a>
</p>

<p align="center"><img src="assets/demo/geullint-demo.gif" alt="GeulLint 浏览器演示" width="100%"></p>

## 在任何写作场景中使用

先在网页中检查一句话；随着项目扩大，再把同一个检查器接入编辑器、终端和 CI。

| 使用场景 | GeulLint 可以做什么 |
| --- | --- |
| VS Code | 输入时实时检查，并一键应用保守修复 |
| CLI | 用一条命令批量检查多个文档或整个仓库 |
| CI | 自动阻止文档质量回退并生成 SARIF 结果 |
| 自定义词汇 | 添加用户词典、dictionary overlay 和项目 rule pack |

当前版本为 **v0.2.0-alpha.1**，包含 100 条核心规则。文本、诊断和遥测不会发送到外部服务。

## 为什么选择 GeulLint

GeulLint 是完全在浏览器、编辑器和终端中**离线**运行的韩语拼写与语法检查器。同一个 Rust 引擎会在各处提供一致的规则 ID 和修复结果。它检查 Markdown 正文以及 JavaScript、TypeScript、Python、Rust 注释，不修改代码和字符串字面量。

| 能力 | 内容 |
| --- | --- |
| 隐私 | 文本、诊断和遥测的网络请求为 0 |
| 开放规则 | 稳定 ID、说明、示例、严重级别和测试 |
| 自动化 | 人类可读输出、JSON、SARIF 2.1.0、LSP、退出码 |
| 自定义 | 用户词典、dictionary overlay、本地 rule pack |
| 平台 | Windows、macOS、Linux，支持 x64 与 ARM64 |

## 安装

无需安装即可使用[本地 WebAssembly 演练场](https://binibinibin123.github.io/geullint/)。

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

脚本会验证 GitHub Release 的 SHA-256。执行前可阅读 [install.ps1](install.ps1) 或 [install.sh](install.sh)。如果已经安装 Rust：

```bash
cargo install --git https://github.com/binibinibin123/geullint --tag v0.2.0-alpha.1 --locked geullint-cli
```

手动压缩包是 [GitHub Releases](https://github.com/binibinibin123/geullint/releases)提供的 fallback。

## 规则与质量

GeulLint v0.2.0-alpha.1 当前提供 **100 条已验证的核心规则**。新审核的42条词汇规则包含84个不同错误句和42个正常反例。在KoLLA v2中由所有标注者判定为正常的249个句子上，本alpha目录出现0个误报。该测试规模较小且只包含正常句，因此不构成precision或recall声明；详见[alpha质量报告](docs/quality-report-v0.2.0-alpha.1.md)。

参阅[全部规则](docs/rules.md)、[质量门槛](docs/quality.md)、[语料评估](docs/corpus-evaluation.md)与[离线策略](docs/offline.md)。

## VS Code

<p align="center"><img src="assets/screenshots/vscode.png" alt="VS Code 中的 GeulLint 诊断、快速修复和规则搜索" width="100%"></p>

从 [Releases](https://github.com/binibinibin123/geullint/releases)下载与平台匹配的 VSIX，然后选择 `Extensions: Install from VSIX...`。本地语言服务器已包含在内，无需 Rust、Node.js 或 API 密钥。

## 限制与贡献

GeulLint 是保守的规则型检查器，不能替代长上下文语义判断或创作编辑。只有在语料具备许可证、哈希和可审查记录时，项目才会公布独立指标。

欢迎阅读 [CONTRIBUTING.md](CONTRIBUTING.md)、[ARCHITECTURE.md](ARCHITECTURE.md)和 [ROADMAP.md](ROADMAP.md)。安全问题请按 [SECURITY.md](SECURITY.md)私下报告。

MIT 许可证。第三方许可见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
