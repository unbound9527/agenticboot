# AI 编程工具安装指南

> 本目录收录 AgenticBoot 支持安装的所有 AI 编程工具的详细安装教程。无论你是开发者还是普通用户，都能通过本指南独立完成安装。

## 支持的工具

| 工具 | 类型 | 安装方式 | Windows | macOS | Linux |
|------|------|---------|---------|-------|-------|
| [Node.js](./nodejs.md) | 运行时依赖 | 下载安装包 | ✅ | ✅ | ✅ |
| [Git](./git.md) | 版本控制 | 下载安装包 | ✅ | ✅ | ✅ |
| [Claude Code](./claude-code.md) | AI 编程 CLI | npm 全局安装 | ✅ | ✅ | ✅ |
| [Codex](./codex.md) | AI 编程 CLI | npm 全局安装 | ✅ | ✅ | ✅ |
| [Gemini CLI](./gemini-cli.md) | AI 编程 CLI | npm 全局安装 | ✅ | ✅ | ✅ |
| [OpenCode](./opencode.md) | AI 编程 CLI | GitHub 二进制 | ❌ | ✅ | ✅ |
| [OpenClaw](./openclaw.md) | 个人 AI 助手 | npm 全局安装 | ⚠️ | ✅ | ✅ |

## 安装前置要求

使用 AgenticBoot 安装工具前，请确保：

1. **网络连接正常** — 能够访问 GitHub 和 npm
2. **Windows 用户** — 需要管理员权限（右键"以管理员身份运行"）
3. **PATH 环境变量** — 安装后 AgenticBoot 会自动将工具目录加入 PATH

## 常见问题

**Q: 安装完成后提示找不到命令？**
- 关闭当前终端窗口，重新打开一个
- 或重启电脑让 PATH 环境变量生效

**Q: npm 安装失败？**
- 检查 Node.js 是否已正确安装：`node --version`
- 尝试更换 npm 镜像源：`npm config set registry https://registry.npmmirror.com`

**Q: GitHub 下载失败？**
- 可能受到网络限制，参考 [网络故障排查](../network-troubleshooting.md)

## 各工具详细介绍

### Node.js
JavaScript 运行时环境，是大多数 CLI 工具的依赖。AgenticBoot 会自动安装包含 npm 的完整版本。

### Git
版本控制系统，用于代码管理。部分 AI 工具依赖 Git 进行版本检测。

### Claude Code
Anthropic 官方 AI 编程助手，支持 Claude 系列模型。提供智能代码补全、文件编辑、任务自动化等功能。

### Codex
OpenAI 官方 AI 编程助手，基于 GPT-4o 驱动，支持代码生成和补全。

### Gemini CLI
Google 官方 Gemini CLI 工具，接入 Gemini 系列模型，支持网络搜索和工具调用。

### OpenCode
开源 AI 编程 CLI 工具（Go 编写），支持 75+ 模型提供商。**注意：目前仅支持 macOS 和 Linux，Windows 版本开发中。**

### OpenClaw
个人 AI 助手，支持多平台消息集成（WhatsApp、Telegram、Discord 等）。**注意：这不是专门的编程工具，而是个人助手工具。**
