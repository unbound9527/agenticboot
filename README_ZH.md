# AgenticBoot

AgenticBoot 是一个以 Windows 为优先的一键装机工具，用来安装和管理 AI 编程相关工具。它的核心目标不是“无脑重装”，而是先检测本机现状，能复用就复用，缺什么再装什么。

当前这条分支主要完成的是 Windows 真实装机链路。macOS 和 Linux 目前只有框架和占位实现，还没有做完正式安装逻辑。

## 当前 Windows 已实现的能力

- 安装前先检测本机是否已经有可用的 `Node.js`、`Git`、CLI 工具或桌面应用。
- 对已经可用的工具直接跳过，不重复安装。
- 支持真正安装官方桌面应用：
  - Claude Desktop
  - Codex 桌面应用
  - OpenCode 桌面应用
- 支持安装以下 CLI / 工具：
  - Claude Code
  - Codex CLI
  - Gemini CLI
  - OpenCode CLI
  - OpenClaw
  - Hermes
- Hermes 在 Windows 下不依赖用户预装 Python，会自动下载一份托管 Python 运行时到 Hermes 目录内再完成安装。
- 卸载时只会自动清理 AgenticBoot 自己托管的目录，不会随意删除系统里原本已有的安装。

## 关键行为说明

### 1. 检测优先，而不是强制重装

这套逻辑不只是检查 Agent 本体，也会检查依赖项。

- 如果机器里已经有可用的 `Node.js`，就直接复用。
- 如果已经有可用的 `Git`，就直接复用。
- 如果某个 CLI 已经可以正常使用，就跳过安装。
- 如果桌面应用已经装在系统其他位置，也会显示为“已安装”，但不会误导用户去点一个并不安全的“卸载”。

### 2. 区分“托管安装”和“外部安装”

AgenticBoot 会区分两类安装来源：

- 托管安装：安装在用户选择的安装根目录下面，由 AgenticBoot 自己创建。
- 外部安装：系统里原本就存在，或者通过其他方式安装的工具。

只有托管安装，才会在卸载时自动删目录。

## 各工具的当前口径

- OpenCode CLI：Windows 下走原生 npm 包 `opencode-ai`，不依赖 WSL。
- OpenClaw：Windows 下走官方 PowerShell 安装路径。
- Hermes：Windows 下自动下载官方 Python 3.11 ZIP 运行时，从 GitHub 拉取 `hermes-agent` 源码包并在本地 `venv` 中按官方安装脚本的分层策略 `pip install`（含 Web 仪表盘依赖），不依赖用户本机已有 Python / `git` / `winget`。

## 当前平台进度

- Windows：已实现核心安装逻辑
- macOS：只有框架
- Linux：只有框架

## 相关文档

- 工具文档索引：[docs/tools/README.md](./docs/tools/README.md)
- Windows 设计文档：[docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md](./docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md)
- 实施计划：[docs/superpowers/plans/2026-05-08-windows-one-click-install.md](./docs/superpowers/plans/2026-05-08-windows-one-click-install.md)
