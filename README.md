# AgenticBoot

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-brightgreen" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/status-active-success" alt="Status">
</p>

<p align="center">
  <strong>🔥 One-click bootstrap your AI coding environment. Zero to coding in 5 minutes.</strong>
</p>

<p align="center">
  English | <a href="#中文">中文</a>
</p>

---

## What is AgenticBoot?

AgenticBoot is a **one-click installer and launcher** for the entire agentic coding ecosystem. It detects your environment, installs the tools you select, injects API provider configs, and gets you coding — all in one flow.

### Supported AI Coding CLI Tools

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
- **China-network optimized** — Mirror fallback for npm, GitHub, and official install scripts. No more `raw.githubusercontent.com` timeouts.
- **Works on your machine** — Windows 10/11 first, macOS and Linux supported too.
- **Open source, MIT** — Inspect every line. No telemetry, no lock-in.

---

## Quick Start

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/unbound9527/agenticboot/main/install.ps1 | iex
```

### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/unbound9527/agenticboot/main/install.sh | bash
```

Then run `agenticboot` and follow the guided setup.

---

## What It Does

```
┌─────────────────────────────────────────┐
│  ① Detect Environment                    │
│  Node.js · Git · npm · Network status    │
├─────────────────────────────────────────┤
│  ② Select Tools                          │
│  ☑ Claude Code  ☑ Codex  ☐ OpenCode     │
│  ☑ OpenClaw     ☐ Hermes ☐ Gemini CLI   │
├─────────────────────────────────────────┤
│  ③ Choose API Provider                   │
│  ☑ Recommended relay  ☐ Custom endpoint  │
├─────────────────────────────────────────┤
│  ④ Install & Configure                   │
│  Download → Install → Inject configs     │
├─────────────────────────────────────────┤
│  ⑤ Ready                                 │
│  All tools installed and configured.     │
│  Open terminal, type your first command. │
└─────────────────────────────────────────┘
```

---

## Features

- **Environment auto-detection** — Checks Node.js, Git, npm, network connectivity before installing
- **Multi-source downloads** — Primary + mirror fallback for every download URL
- **Config injection** — Writes API endpoint, key, and model settings for each tool automatically
- **Provider presets** — Built-in relay provider configs, one-click apply
- **Non-destructive** — Won't overwrite existing configs without asking
- **Uninstall support** — Clean removal of installed tools when needed

---

## Roadmap

- [ ] GUI launcher (Tauri desktop app)
- [ ] Built-in relay speed test & provider recommendation
- [ ] One-click recharge integration for relay providers
- [ ] Tool update manager
- [ ] Team/org config sync

---

## Contributing

Issues and PRs welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## License

MIT © [YiYun Zhang](https://github.com/unbound9527)

---

<a name="中文"></a>

# AgenticBoot 中文说明

## 这是什么？

**AgenticBoot** 是一个 AI 编程 CLI 工具的**一键装机向导**。自动检测你的电脑环境，批量安装 Claude Code、Codex、OpenCode、OpenClaw、Hermes 等主流 AI 编程工具，并帮你配置好 API 中转站，装完即用。

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

### 特色

- **国内网络优化** — GitHub、npm 下载自动切换镜像源
- **环境检测** — Node.js、Git、网络连通性一次查完
- **配置注入** — 自动写入各工具的 API endpoint 和密钥配置
- **非破坏性** — 已有配置不会被覆盖
- **开源（MIT）** — 所有代码可审查，无遥测、无捆绑

### 快速开始

#### Windows
```powershell
irm https://raw.githubusercontent.com/unbound9527/agenticboot/main/install.ps1 | iex
```

#### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/unbound9527/agenticboot/main/install.sh | bash
```

然后运行 `agenticboot`，按引导操作即可。

### 开发路线图

- [ ] GUI 桌面启动器（Tauri）
- [ ] 内置中转站测速 & 推荐
- [ ] 中转站一键充值集成
- [ ] 工具版本更新管理
- [ ] 团队配置同步

### 贡献

欢迎提 Issue 和 PR。

### 许可证

MIT © [YiYun Zhang](https://github.com/unbound9527)
