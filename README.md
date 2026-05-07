# AgenticBoot

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-brightgreen" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
</p>

<p align="center">
  <strong>One-click bootstrap your AI coding environment. Zero to coding in 5 minutes.</strong>
</p>

<p align="center">
  <a href="./README_ZH.md">中文</a>
</p>

> ⚡ First release coming soon. Full docs and install scripts are being prepared.

---

## What is AgenticBoot?

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

---

## Features

- **Environment auto-detection** — Checks Node.js, Git, npm, network connectivity before installing
- **Multi-source downloads** — Primary + mirror fallback for every download URL
- **Config injection** — Writes API endpoint, key, and model settings for each tool automatically
- **Provider presets** — Built-in relay provider configs, one-click apply
- **Non-destructive** — Won't overwrite existing configs without asking

---

## Roadmap

- [ ] GUI launcher (Tauri desktop app)
- [ ] Built-in relay speed test & provider recommendation
- [ ] One-click recharge integration for relay providers
- [ ] Tool update manager
- [ ] Team/org config sync

---

## License

MIT © YiYun Zhang
