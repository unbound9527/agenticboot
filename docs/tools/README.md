# AI 编程工具安装指南

> 本目录收录所有支持的 AI 编程工具的独立安装教程。不使用 AgenticBoot 时，可参照本目录完成手动安装。

## 目录

| 工具 | 安装方式 |
|------|---------|
| [Node.js](./nodejs.md) | 下载安装包 / 包管理器 |
| [Git](./git.md) | 下载安装包 / 包管理器 |
| [Claude Code](./claude-code.md) | npm 全局安装 |
| [Codex](./codex.md) | npm 全局安装 |
| [Gemini CLI](./gemini-cli.md) | npm 全局安装 |
| [OpenCode](./opencode.md) | 安装脚本 / 二进制下载 |
| [OpenClaw](./openclaw.md) | npm 全局安装 |

## 共同前提

| 系统 | 必需条件 |
|------|---------|
| Windows | PowerShell 或 CMD |
| macOS | Terminal |
| Linux | Terminal + curl |
| **全部** | Node.js（除 Node.js 和 Git 本身外） |

## 通用卸载命令

```bash
# 卸载 npm 全局包
npm uninstall -g <包名>

# macOS/Linux 同时删除配置
rm -rf ~/.npmrc
```

## 网络说明

如遇 GitHub/npm 下载失败，参考 [网络故障排查](../network-troubleshooting.md)
