# AgenticBoot 协作说明

## 项目定位

这是一个基于 CC Switch 的增量二开仓库。当前协作重点不是重新理解整套 CC Switch，而是优先处理 AgenticBoot 新增或重度改造的装机、安装检测、工具管理相关能力。

默认目标：快速进入当前二开主线，减少无关阅读和 token 消耗。

## 阅读顺序

开始任务时，默认按这个顺序建立上下文：

1. 先看 `README.md`，只获取项目一句话定位和当前目标。
2. 再看当前任务直接相关的二开主战场文件。
3. 只有当任务明确涉及上游继承逻辑时，再扩展到 CC Switch 原有模块。

如果只是修复安装、检测、装机流程、工具卡片、安装日志、安装进度，不要先大范围扫描整个仓库。

## 当前主战场

优先关注这些目录和文件：

- Rust 安装链路：`src-tauri/src/services/installer/`
- 工具插件与检测：`src-tauri/src/plugins/`
- 工具类型与插件注册：`src-tauri/src/tool_types.rs`、`src-tauri/src/plugin.rs`
- 前端装机/管理页面：`src/pages/Wizard.tsx`、`src/pages/Manager.tsx`
- 工具相关组件：`src/components/tools/`
- 安装状态相关 hooks：`src/hooks/useInstallProgress.ts`、`src/hooks/useInstallSessions.ts`
- 安装链路测试：`tests/components/Manager.installDetection.test.tsx`、`tests/components/Wizard.installDetection.test.tsx`、`tests/lib/installSessions.test.ts`

如果需要理解产品意图，再补看：

- `docs/superpowers/specs/`
- `docs/superpowers/plans/`

## 非必要少看

以下内容默认不要主动深读，除非当前任务直接相关：

- `cc-switch-main/` 下的上游镜像内容
- 大部分多语言用户文档：`docs/user-manual/`
- 与安装器无关的大块上游通用页面、Provider、Proxy、Usage 逻辑
- 仅用于资源展示的 `assets/` 大文件

原则不是禁止查看，而是先把注意力放在 AgenticBoot 的二开部分。

## 修改原则

- 优先做增量修改，不做“顺手整理”式的大范围重构。
- 若任务涉及安装、检测、日志、状态同步，先检查对应测试是否已有覆盖。
- 若发现逻辑像是继承自 CC Switch，先确认它是否真的影响当前二开目标，再决定是否继续深挖。
- 处理问题时，优先从最近活跃改动文件入手，不要先扩散式阅读全仓。

## 常用命令

- `pnpm typecheck`
- `pnpm test:unit`
- `pnpm dev`

## 期望的工作方式

回答和实现时优先：

- 快速定位二开主线
- 说明改动是否落在 AgenticBoot 自己的新增能力上
- 只引用完成任务所需的最小上下文

如果任务与上游 CC Switch 原始能力无关，就不要把大量注意力花在上游背景介绍上。
