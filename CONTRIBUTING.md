# Contributing to AgenticBoot

> [中文版本](#贡献指南)

AgenticBoot is a derivative project based on CC Switch. Thank you for your interest in contributing!

Please read our [Code of Conduct](./CODE_OF_CONDUCT.md) before participating.

## Project Background

AgenticBoot is a fork of [CC Switch](https://github.com/farion1231/cc-switch) with significant modifications including Hermes Desktop integration, Windows path corrections, and enhanced tool management. Contributions should align with AgenticBoot's scope as an AI developer tool manager.

## How to Contribute

There are many ways to contribute:

- **Report bugs** — Found something broken? [Open a bug report](https://github.com/unbound9527/agenticboot/issues/new).
- **Suggest features** — Have an idea? [Submit a feature request](https://github.com/unbound9527/agenticboot/issues/new).
- **Improve docs** — Spot a typo or missing info? Open a PR or issue.
- **Contribute code** — Fix bugs or implement features via pull requests.

> **Security vulnerabilities**: Please do NOT use public issues. See our [Security Policy](./SECURITY.md) instead.

## Development Setup

### Prerequisites

- Node.js 18+ and pnpm 8+
- Rust 1.85+ and Cargo
- [Tauri 2.0 prerequisites](https://v2.tauri.app/start/prerequisites/)

### Quick Start

```bash
# Install dependencies
pnpm install

# Start development server with hot reload
pnpm dev
```

### Useful Commands

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start dev server (hot reload) |
| `pnpm build` | Production build |
| `pnpm typecheck` | TypeScript type checking |
| `pnpm test:unit` | Run unit tests |
| `pnpm lint` | ESLint check |
| `pnpm format` | Format code (Prettier) |
| `pnpm format:check` | Check code formatting |

For Rust backend:

```bash
cd src-tauri
cargo fmt        # Format Rust code
cargo clippy     # Run linter
cargo test       # Run tests
```

## Code Style

- **Frontend**: Prettier for formatting, ESLint for linting, strict TypeScript (`pnpm typecheck`)
- **Backend**: `cargo fmt` for formatting, `cargo clippy` for linting
- **Tauri 2.0**: Command names must use camelCase

Run all checks before submitting:

```bash
pnpm typecheck && pnpm format:check && pnpm test:unit
cd src-tauri && cargo fmt --check && cargo clippy && cargo test
```

## Pull Request Guidelines

1. **Open an issue first** for new features — PRs for features that are not a good fit may be closed.
2. **Fork and branch** — Create a feature branch from `main` (e.g., `feat/my-feature` or `fix/issue-123`).
3. **Keep PRs focused** — One feature or fix per PR. Avoid unrelated changes.
4. **Upstream awareness** — If your change addresses a bug also present in upstream CC Switch, consider submitting a fix there as well.

### PR Checklist

- [ ] `pnpm typecheck` passes
- [ ] `pnpm format:check` passes
- [ ] `cargo clippy` passes (if Rust code changed)

### Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(provider): add support for new provider
fix(tray): resolve menu not updating after switch
docs(readme): update installation instructions
ci: add format check workflow
chore(deps): update dependencies
```

## Questions?

- [Open an issue](https://github.com/unbound9527/agenticboot/issues/new)
- [GitHub Discussions](https://github.com/unbound9527/agenticboot/discussions)

---

# 贡献指南

> [English Version](#contributing-to-agenticboot)

AgenticBoot 是基于 CC Switch 的二开项目。感谢你的贡献兴趣！

参与之前请阅读我们的[行为准则](./CODE_OF_CONDUCT.md)。

## 项目背景

AgenticBoot 是 [CC Switch](https://github.com/farion1231/cc-switch) 的分支，在此基础上做了大量修改，包括 Hermes Desktop 集成、Windows 路径修正和增强的工具管理。贡献应契合 AgenticBoot 作为 AI 开发者工具管理器的定位。

## 如何贡献

你可以通过多种方式参与贡献：

- **报告 Bug** — 发现问题？[提交 Bug 报告](https://github.com/unbound9527/agenticboot/issues/new)。
- **建议功能** — 有想法？[提交功能请求](https://github.com/unbound9527/agenticboot/issues/new)。
- **改进文档** — 发现错误或缺失？直接提 PR 或 Issue。
- **贡献代码** — 通过 Pull Request 修复 Bug 或实现新功能。

> **安全漏洞**：请不要使用公开 Issue 报告。请参阅我们的[安全策略](./SECURITY.md)。

## 开发环境搭建

### 前提条件

- Node.js 18+ 和 pnpm 8+
- Rust 1.85+ 和 Cargo
- [Tauri 2.0 开发环境](https://v2.tauri.app/start/prerequisites/)

### 快速开始

```bash
# 安装依赖
pnpm install

# 启动开发服务器（热重载）
pnpm dev
```

### 常用命令

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 启动开发服务器（热重载） |
| `pnpm build` | 构建生产版本 |
| `pnpm typecheck` | TypeScript 类型检查 |
| `pnpm test:unit` | 运行单元测试 |
| `pnpm lint` | ESLint 检查 |
| `pnpm format` | 格式化代码（Prettier） |
| `pnpm format:check` | 检查代码格式 |

Rust 后端命令：

```bash
cd src-tauri
cargo fmt        # 格式化 Rust 代码
cargo clippy     # 运行 Clippy 检查
cargo test       # 运行测试
```

## 代码规范

- **前端**：使用 Prettier 格式化、ESLint 检查、严格 TypeScript（`pnpm typecheck`）
- **后端**：使用 `cargo fmt` 格式化、`cargo clippy` 检查
- **Tauri 2.0**：命令名必须使用 camelCase

提交前运行所有检查：

```bash
pnpm typecheck && pnpm format:check && pnpm test:unit
cd src-tauri && cargo fmt --check && cargo clippy && cargo test
```

## Pull Request 指南

1. **先开 Issue 讨论** — 新功能请先开 Issue，不适合项目方向的 PR 可能会被关闭。
2. **Fork 并创建分支** — 从 `main` 创建功能分支（如 `feat/my-feature` 或 `fix/issue-123`）。
3. **保持 PR 专注** — 每个 PR 只做一件事，避免无关改动。
4. **关注上游** — 如果你的修改解决了上游 CC Switch 也存在的 bug，建议同时向上游提交修复。

### PR 检查清单

- [ ] `pnpm typecheck` 通过
- [ ] `pnpm format:check` 通过
- [ ] `cargo clippy` 通过（如修改了 Rust 代码）

### 提交信息规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat(provider): add support for new provider
fix(tray): resolve menu not updating after switch
docs(readme): update installation instructions
ci: add format check workflow
chore(deps): update dependencies
```

## 有疑问？

- [提交 Issue](https://github.com/unbound9527/agenticboot/issues/new)
- [GitHub 讨论区](https://github.com/unbound9527/agenticboot/discussions)
