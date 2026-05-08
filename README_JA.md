<div align="center">

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
  <a href="README.md">English</a> | <a href="README_ZH.md">中文</a> | 日本語
</p>

> 🚧 **Coming soon.** Full documentation, install scripts, and first release are being prepared. Stay tuned.

</div>

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

---

## Features

### Provider Management

- One-click provider import/export
- 50+ built-in presets for popular relay services
- Drag-and-drop reordering
- Universal provider sync across apps

### Local Proxy & Failover

- Hot-switch between providers without restarting CLI
- Automatic failover on provider errors
- Per-app routing (Claude, Codex, Gemini independently)
- Circuit breaker and health monitoring

### MCP, Prompts & Skills

- Unified MCP panel across all supported apps
- Cross-app prompt sync (CLAUDE.md / AGENTS.md / GEMINI.md)
- One-click skill install from GitHub repos
- Symbolic link or file copy deployment

### Usage Tracking

- Cross-provider cost and token tracking
- Trend charts and detailed request logs
- Custom model pricing configuration

### Session Manager

- Browse, search, and restore conversations across all apps
- Workspace editor for OpenClaw agents

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React + TS)                    │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Components  │  │    Hooks     │  │  TanStack Query  │  │
│  │   (UI)      │──│ (Bus. Logic) │──│   (Cache/Sync)   │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │ Tauri IPC
┌────────────────────────▼────────────────────────────────────┐
│                  Backend (Tauri + Rust)                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Commands   │  │   Services   │  │  Models/Config    │  │
│  │ (API Layer) │──│ (Bus. Layer) │──│     (Data)       │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

**Frontend**: React 18 · TypeScript · Vite · Tailwind CSS · TanStack Query v5

**Backend**: Tauri 2 · Rust · SQLite · tokio

**UI Components**: shadcn/ui · Radix UI · Lucide Icons

---

## License

MIT © [unbound9527](https://github.com/unbound9527)
