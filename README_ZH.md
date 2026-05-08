# AgenticBoot

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-brightgreen" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/status-active-success" alt="Status">
</p>

<p align="center">
  <strong>🔥 一键搭建 AI 编程环境，5 分钟从零到开始写代码。</strong>
</p>

<p align="center">
  <a href="README.md">English</a> | 中文
</p>

> 🚧 **即将上线。** 完整文档、安装脚本和首个版本正在准备中。

---

## AgenticBoot 是什么？

**AgenticBoot** 是一个 AI 编程 CLI 工具的**一键装机向导**。自动检测你的电脑环境，批量安装 Claude Code、Codex、OpenCode、OpenClaw、Hermes 等主流 AI 编程工具，并帮你配置好 API 中转站，装完即用。

### 支持的 AI 编程 CLI 工具

| 工具 | 说明 |
|------|------|
| **Claude Code** | Anthropic 官方 CLI 编程助手 |
| **Codex** | OpenAI 官方 CLI 编程助手 |
| **OpenCode** | 开源编程助手 |
| **OpenClaw** | 无头可编程编程引擎 |
| **Hermes** | 多供应商 AI 编程助手 |
| **Gemini CLI** | Google 官方 CLI 编程助手 |

### 为什么选择 AgenticBoot？

- **一个安装器搞定所有工具** — 不用再跑 5 条不同的安装命令，勾选、点击、完成。
- **预配置供应商** — 内置主流 API 中转站预设配置，无需手动编辑 `settings.json`。
- **国内网络优化** — npm、GitHub、官方安装脚本均支持镜像回退，告别 `raw.githubusercontent.com` 超时。
- **跨平台** — Windows 10/11 优先，同时支持 macOS 和 Linux。
- **开源（MIT）** — 代码可审查，无遥测、无锁定。

---

## 快速开始

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/unbound9527/agenticboot/main/install.ps1 | iex
```

### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/unbound9527/agenticboot/main/install.sh | bash
```

然后运行 `agenticboot`，按引导操作即可。

---

## 工作流程

```
┌─────────────────────────────────────────┐
│  ① 检测环境                               │
│  Node.js · Git · npm · 网络连通性          │
├─────────────────────────────────────────┤
│  ② 选择工具                               │
│  ☑ Claude Code  ☑ Codex  ☐ OpenCode     │
│  ☑ OpenClaw     ☐ Hermes ☐ Gemini CLI   │
├─────────────────────────────────────────┤
│  ③ 选择 API 供应商                         │
│  ☑ 推荐中转站  ☐ 自定义端点                 │
├─────────────────────────────────────────┤
│  ④ 安装与配置                              │
│  下载 → 安装 → 注入配置                     │
├─────────────────────────────────────────┤
│  ⑤ 完成                                   │
│  所有工具已安装配置完毕，打开终端即可开始编码。  │
└─────────────────────────────────────────┘
```

---

## 功能特性

- **环境自动检测** — 安装前检查 Node.js、Git、npm、网络连通性
- **多源下载** — 每个下载 URL 均支持主源 + 镜像回退
- **配置注入** — 自动为各工具写入 API 端点、密钥和模型配置
- **供应商预设** — 内置中转站配置，一键应用
- **非破坏性** — 不会在未确认的情况下覆盖已有配置
- **卸载支持** — 支持干净卸载已安装的工具

---

## 开发路线图

- [ ] GUI 桌面启动器（Tauri）
- [ ] 内置中转站测速 & 推荐
- [ ] 中转站一键充值集成
- [ ] 工具版本更新管理
- [ ] 团队配置同步

---

## 贡献

欢迎提 Issue 和 PR。详见 [CONTRIBUTING.md](./CONTRIBUTING.md)。

---

## 许可证

MIT © [unbound9527](https://github.com/unbound9527)
