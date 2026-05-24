# AgenticBoot

[中文](#中文) | [English](#english)

## 中文

### AgenticBoot 是什么

AgenticBoot 是一个面向 Windows 的 AI 编码工具装机与管理器。

它的目标不是再做一个“工具列表”，而是把 AI 编码环境最麻烦的一段先解决掉：先检测你机器上已经能用的东西，避免重复安装；再把你真正需要的工具补齐；最后把安装、卸载、状态查看和后续管理统一到一个入口里。

### 愿景

我们希望 AgenticBoot 最终能成为一个更省心的 AI coding bootstrapper：

- 新机器装环境时，不需要手动拼命查文档、装依赖、踩 Windows 坑
- 老机器补工具时，不需要因为“已经装过一部分”就把环境越搞越乱
- 多个 AI CLI、桌面端和依赖工具可以在一个地方被发现、安装、管理和清理
- 安装过程不是黑箱，用户能看见检测结果、安装进度、关键日志和最终状态

### 它想解决的核心问题

今天很多 AI 编码工具都默认用户已经准备好了 Node.js、Git、Python、npm、PowerShell、系统 PATH，或者默认用户愿意自己处理每个工具的 Windows 差异。

AgenticBoot 想把这些碎片化步骤收束成一个更统一的体验：

- 检测优先，而不是默认重装
- 尽量复用已有可用环境，而不是强行接管
- 对外部已安装工具和 AgenticBoot 自己管理的工具做统一视图
- 在 Windows 上优先走真实可用的官方安装路径，而不是只停留在占位支持

### 现在你能得到什么

当前这一阶段，项目重点非常明确：**先把 Windows 装机链路做扎实。**

目前已经落地的价值包括：

- 安装前自动检测 `Node.js`、`Git` 和已支持 AI 工具是否已可用
- 已有可用安装时跳过重复安装，减少时间浪费和环境污染
- 能在统一管理页中识别 AgenticBoot 管理的安装和系统里原本就存在的外部安装
- 提供统一的卸载管理流程，而不是要求用户回到每个工具各自的卸载方式里摸索
- 最近一轮改动重点增强了安装过程反馈，让 `Wizard` 和 `Manager` 两条主流程里都能更连续地看到活动状态、进度变化和日志上下文

### 当前现状

AgenticBoot 目前是 **Windows-first** 项目。

- Windows：核心装机、检测、安装管理链路已在持续实现中，也是当前主战场
- macOS：已有框架和占位，但还不是完整可用的安装实现
- Linux：已有框架和占位，但还不是完整可用的安装实现

这意味着如果你今天来体验 AgenticBoot，最值得关注的是它在 Windows 上对真实安装流程的处理，而不是跨平台完整度。

### 当前已支持的 Windows 能力

#### 依赖检测与复用

- 检测并复用已可用的 `Node.js`
- 检测并复用已可用的 `Git`
- 检测已存在的 CLI / 桌面工具，避免不必要重装

#### 桌面应用安装

- Claude Desktop
- Codex desktop app
- OpenCode desktop app

#### CLI 工具安装

- Claude Code
- Codex CLI
- Gemini CLI
- OpenCode CLI
- OpenClaw
- Hermes

#### 工具级安装策略

- OpenCode CLI 在 Windows 上走原生 `opencode-ai` npm 包，不依赖 WSL
- OpenClaw 在 Windows 上走官方 PowerShell 安装路径
- Hermes 在 Windows 上可由 AgenticBoot 自行拉起受管 Python 运行时和本地 `venv`，不要求用户先手装 Python

### AgenticBoot 如何看待“已安装”

AgenticBoot 区分两类安装：

- **Managed installs**：安装在 AgenticBoot 选定目录下，由 AgenticBoot 直接管理
- **External installs**：系统里原本就存在的安装，由 AgenticBoot 检测到并接入统一视图

这带来的好处是：

- 你已经装过的工具可以直接复用
- AgenticBoot 不会轻易冒充自己“拥有”系统外部目录
- 只有受管目录里的内容才会进入自动清理边界，降低误删风险

### 后续预计推进

接下来项目大概率会继续沿着这条主线推进：

- 继续完善 Windows 安装链路的稳定性、检测准确度和卸载一致性
- 继续打磨 `Wizard` 与 `Manager` 的装机反馈体验，让状态、日志和结果更直观
- 扩展更多工具的真实可用安装与检测能力
- 在 Windows 主链路稳定后，再逐步补齐 macOS / Linux 的非占位实现

### 适合谁关注这个项目

如果你符合下面任一类场景，AgenticBoot 会比较值得关注：

- 你想在 Windows 上更快搭起 AI 编码环境
- 你不想为每个 AI 工具单独处理依赖和安装细节
- 你已经装过部分工具，但还希望统一查看和管理状态
- 你更在意“真实可用的装机流程”，而不是只有概念上的跨平台支持

### 开发启动

如果你要在本地启动桌面应用，建议直接使用仓库自带脚本。它会按仓库声明的 Node.js / `pnpm` 版本准备运行环境。

```powershell
.\scripts\dev-desktop.ps1
```

或：

```bat
scripts\dev-desktop.cmd
```

常用命令：

```bash
pnpm typecheck
pnpm test:unit
pnpm dev
```

### 相关文档

- Tool docs: [docs/tools/README.md](./docs/tools/README.md)
- Windows install design: [docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md](./docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md)
- Install activity feedback design: [docs/superpowers/specs/2026-05-11-install-activity-feed-design.md](./docs/superpowers/specs/2026-05-11-install-activity-feed-design.md)
- Implementation plan: [docs/superpowers/plans/2026-05-08-windows-one-click-install.md](./docs/superpowers/plans/2026-05-08-windows-one-click-install.md)

---

## English

### What AgenticBoot Is

AgenticBoot is a Windows-first bootstrapper and manager for AI coding tools.

It is not just a catalog of apps. The current goal is to make the hardest part of setting up an AI coding environment much less painful: detect what is already usable on the machine, avoid redundant installs, install only what is still missing, and keep installation, uninstall, status visibility, and follow-up management in one place.

### Vision

AgenticBoot aims to become a more reliable AI coding bootstrapper for real-world machines:

- setting up a fresh machine should not require stitching together scattered docs and Windows-specific workarounds
- adding one more tool to an existing machine should not turn into duplicate installs and path conflicts
- AI CLIs, desktop apps, and their dependencies should be discoverable and manageable from one entry point
- installation should not feel like a black box; users should be able to see detection results, progress, meaningful logs, and the final state

### The Problem It Tries To Solve

Many AI coding tools still assume the user already has Node.js, Git, Python, npm, PowerShell, PATH setup, and enough patience to navigate each tool's Windows-specific quirks.

AgenticBoot tries to turn that fragmented experience into a more unified one:

- detection first instead of reinstall first
- reuse working local environments instead of taking ownership by force
- show AgenticBoot-managed installs and externally detected installs in one management flow
- prefer real, working Windows install paths over placeholder support

### What You Can Get Today

At this stage, the project is intentionally focused: **make the Windows install flow solid first.**

What is already valuable today:

- automatic detection of usable `Node.js`, `Git`, and supported AI tools before installation
- skipping redundant installs when a working local installation already exists
- a unified management view for both AgenticBoot-managed installs and tools that were installed outside AgenticBoot
- a unified uninstall flow instead of forcing users to rediscover each tool's original uninstall path
- recent work has improved install feedback in both `Wizard` and `Manager`, making activity, progress, and log context feel more continuous during installs

### Current Status

AgenticBoot is currently a **Windows-first** project.

- Windows: the core detection, install, and management flows are actively implemented and are the main focus
- macOS: framework and scaffolding exist, but the install path is not fully implemented yet
- Linux: framework and scaffolding exist, but the install path is not fully implemented yet

If you are evaluating the project today, the most accurate way to read it is as a Windows-focused product in active build-out, not as a fully finished cross-platform installer.

### Current Windows Support

#### Dependency Detection And Reuse

- detect and reuse working `Node.js`
- detect and reuse working `Git`
- detect existing CLI and desktop installations to avoid unnecessary reinstalls

#### Desktop App Installs

- Claude Desktop
- Codex desktop app
- OpenCode desktop app

#### CLI Installs

- Claude Code
- Codex CLI
- Gemini CLI
- OpenCode CLI
- OpenClaw
- Hermes

#### Tool-Specific Notes

- OpenCode CLI uses the native Windows `opencode-ai` npm package and does not depend on WSL
- OpenClaw follows the official PowerShell install path on Windows
- Hermes can install with an AgenticBoot-managed Python runtime and local `venv`, so users do not need to preinstall Python first

### How AgenticBoot Treats Existing Installs

AgenticBoot distinguishes between two kinds of installs:

- **Managed installs**: created under the install root selected for AgenticBoot
- **External installs**: already present elsewhere on the system and detected by AgenticBoot

That distinction matters because it lets AgenticBoot:

- reuse tools you already have
- avoid pretending it owns unrelated system directories
- keep automatic cleanup scoped to the managed install root instead of risking overreach

### What We Expect To Push Next

The near-term direction is likely to stay on this line:

- keep improving Windows install stability, detection accuracy, and uninstall consistency
- keep refining install feedback in `Wizard` and `Manager`, with clearer state, logs, and outcomes
- expand real install and detection coverage across more tools
- fill in macOS and Linux with non-placeholder implementations after the Windows path is solid

### Who This Project Is For

AgenticBoot is especially relevant if you:

- want to get an AI coding environment running faster on Windows
- do not want to manually resolve dependency and install details for every tool
- already have some tools installed and still want a unified view of what is usable
- care more about real install behavior than broad but shallow platform claims

### Development Startup

To run the desktop app locally on Windows, use the repository-managed startup script. It prepares the runtime using the Node.js and `pnpm` versions declared by the repo.

```powershell
.\scripts\dev-desktop.ps1
```

Or:

```bat
scripts\dev-desktop.cmd
```

Common commands:

```bash
pnpm typecheck
pnpm test:unit
pnpm dev
```

### Related Docs

- Tool docs: [docs/tools/README.md](./docs/tools/README.md)
- Windows install design: [docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md](./docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md)
- Install activity feedback design: [docs/superpowers/specs/2026-05-11-install-activity-feed-design.md](./docs/superpowers/specs/2026-05-11-install-activity-feed-design.md)
- Implementation plan: [docs/superpowers/plans/2026-05-08-windows-one-click-install.md](./docs/superpowers/plans/2026-05-08-windows-one-click-install.md)
