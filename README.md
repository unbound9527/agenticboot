# AgenticBoot

<p align="center">
  <a href="#english">English</a> | <a href="#chinese">中文</a>
</p>

---

<a name="english"></a>

## One-click bootstrap your AI coding environment. Zero to coding in 5 minutes.

> ⚡ First release coming soon. Full docs and install scripts are being prepared.

### What is AgenticBoot?

AgenticBoot is a **one-click installer and launcher** for the agentic coding ecosystem. It detects your environment, installs the tools you select, injects API provider configs, and gets you coding — all in one flow.

### Supported Tools

| Tool | Description |
|------|-------------|
| **Claude Code** | Anthropic's official CLI coding agent |
| **Codex** | OpenAI's CLI coding agent |
| **OpenCode** | Open-source coding agent |
| **OpenClaw** | Headless programmable coding engine |
| **Hermes** | Multi-provider AI coding assistant |
| **Gemini CLI** | Google's CLI coding agent |

### Why AgenticBoot?

- **5 tools, 1 installer** — Stop running 5 different install commands. Check the boxes, click install, done.
- **Pre-configured providers** — Built-in presets for popular API relay services. No manual `settings.json` editing.
- **China-network optimized** — Mirror fallback for npm, GitHub, and official install scripts.
- **Windows-first** — Windows 10/11 first, macOS and Linux supported too.
- **MIT licensed** — Inspect every line. No telemetry, no lock-in.

### Features

- **Environment auto-detection** — Checks Node.js, Git, npm, network connectivity before installing
- **Multi-source downloads** — Primary + mirror fallback for every download URL
- **Config injection** — Writes API endpoint, key, and model settings for each tool automatically
- **Provider presets** — Built-in relay provider configs, one-click apply
- **Non-destructive** — Won't overwrite existing configs without asking

### Roadmap

- [ ] GUI launcher (Tauri desktop app)
- [ ] Built-in relay speed test & provider recommendation
- [ ] One-click recharge integration for relay providers
- [ ] Tool update manager
- [ ] Team/org config sync

### License

MIT © YiYun Zhang

---

<a name="chinese"></a>

## AI 编程环境一键装机工具。5 分钟从零到开始写代码。

> ⚡ 首个版本即将发布，完整文档和安装脚本正在准备中。

### 这是什么？

**AgenticBoot** 是一个 AI 编程 CLI 工具的**一键装机向导**。自动检测你的电脑环境，批量安装 Claude Code、Codex、OpenCode、OpenClaw、Hermes、Gemini CLI 等主流 AI 编程工具，并帮你配置好 API 中转站，装完即用。

### 解决什么问题？

每个 AI 编程工具都有自己的安装方式（npm / scoop / winget / 官方脚本），配置文件路径和格式各不相同，国内访问 GitHub 和 npm 还经常连不上。新手从零到用上往往要折腾一两个小时。

AgenticBoot 把它压缩成：**打开 → 勾选 → 等两分钟 → 开始写代码。**

### 支持的工具

| 工具 | 安装方式 |
|------|---------|
| Claude Code | 原生安装器 / npm 镜像回退 |
| Codex | npm 全局安装 |
| OpenCode | GitHub Release / Scoop |
| OpenClaw | npm / GitHub Release |
| Hermes | GitHub Release |
| Gemini CLI | npm 全局安装 |

### 功能

- **环境检测** — Node.js、Git、npm、网络连通性一次查完
- **多源下载** — GitHub、npm 下载自动切换镜像源
- **配置注入** — 自动写入各工具的 API endpoint 和密钥配置
- **中转站预设** — 内置主流中转站配置，一键应用
- **非破坏性** — 已有配置不会被覆盖

### 开发路线

- [ ] GUI 桌面启动器（Tauri）
- [ ] 内置中转站测速 & 推荐
- [ ] 中转站一键充值集成
- [ ] 工具版本更新管理
- [ ] 团队配置同步

### 协议

MIT © YiYun Zhang
