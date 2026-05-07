# AgenticBoot

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-brightgreen" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
</p>

<p align="center">
  <strong>AI 编程环境一键装机工具。5 分钟从零到开始写代码。</strong>
</p>

<p align="center">
  <a href="./README.md">English</a>
</p>

> ⚡ 首个版本即将发布，完整文档和安装脚本正在准备中。

---

## 这是什么？

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

---

## 功能

- **环境检测** — Node.js、Git、npm、网络连通性一次查完
- **多源下载** — GitHub、npm 下载自动切换镜像源
- **配置注入** — 自动写入各工具的 API endpoint 和密钥配置
- **中转站预设** — 内置主流中转站配置，一键应用
- **非破坏性** — 已有配置不会被覆盖

---

## 开发路线

- [ ] GUI 桌面启动器（Tauri）
- [ ] 内置中转站测速 & 推荐
- [ ] 中转站一键充值集成
- [ ] 工具版本更新管理
- [ ] 团队配置同步

---

## 协议

MIT © YiYun Zhang
