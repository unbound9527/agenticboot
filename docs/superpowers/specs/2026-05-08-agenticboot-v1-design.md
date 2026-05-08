# AgenticBoot v1 Design Spec

> Fork 自 [CC Switch](https://github.com/farion1231/cc-switch) (v3.14.1, MIT)，增量添加 AI 编程工具的一键安装/卸载管理能力。

## 项目定位

**AI 开发者装机管家** — 基于 CC Switch 二次开发，在其配置管理能力之上增加应用安装/卸载/更新管理。像一个"软件管家"，专注于 AI 编程 CLI 工具 + AI IDE + 本地模型工具的安装管理。

## 技术栈

| 层 | 技术 | 说明 |
|---|------|------|
| 桌面框架 | Tauri 2 | 复用 CC Switch 现有配置 |
| 后端 | Rust | 安装引擎、PATH管理、环境检测 |
| 前端 | React 18 + TypeScript + Tailwind | 复用 CC Switch 组件库 |
| UI 组件 | shadcn/ui (Radix) | 已有 |
| 数据库 | SQLite (rusqlite) | 新增 installed_tools 表 |
| 包管理 | pnpm | 已有 |

## 核心功能

### 1. 环境检测

- 启动时检测 Node.js（>= 18）、Git、网络连通性
- 检测不通过时**不内置代理/镜像逻辑**，给出指向外部教程的链接
- 检测结果通过 Tauri command 返回前端，展示在向导页

### 2. 首次向导 (Wizard)

- 首次启动展示，之后可跳过
- 步骤：环境检测 → 选择安装根目录 → 勾选工具 → 一键安装
- 安装根目录统一配置，自动创建子目录（如 `<root>/claude-code/`）
- 安装进度通过 Tauri event 实时推送

### 3. 管家页 (Manager)

- 日常使用的工具列表
- 每个工具显示：图标、名称、版本、状态（已安装/未安装/有新版本/安装中）
- 操作：安装、卸载、更新
- PATH 和 shim 由 Manager 自动管理，用户无需手动配置

### 4. 一键卸载

- 调用插件 uninstall 逻辑
- 删除 shim
- 删除安装目录
- 更新数据库状态

### 5. 支持的初始工具（6 个 AI CLI + 扩展）

**第一期：**
- Claude Code、Codex、Gemini CLI、OpenCode、OpenClaw、Hermes

**第二期扩展：**
- VS Code、Cursor、Windsurf（AI IDE）
- Ollama（本地模型）

**扩展机制：** 通过 Rust trait 定义的插件接口，新增工具只需实现 trait 并注册

## 架构设计

### 项目结构

```
agenticboot/
├── src-tauri/src/
│   ├── commands/tools.rs       ← 新增
│   ├── services/installer.rs   ← 新增
│   ├── database/schema.rs      ← 修改：v11 migration
│   ├── store.rs                ← 修改：AppState
│   └── lib.rs                  ← 修改：setup
├── src/
│   ├── components/tools/       ← 新增
│   │   ├── ToolCard.tsx
│   │   ├── InstallProgress.tsx
│   │   ├── EnvCheckPanel.tsx
│   │   └── PathConfig.tsx
│   ├── pages/
│   │   ├── Wizard.tsx           ← 新增
│   │   └── Manager.tsx          ← 新增
│   ├── plugins/                 ← 新增：工具插件目录
│   │   ├── claude-code/{plugin.json, index.ts}
│   │   ├── codex/{plugin.json, index.ts}
│   │   ├── gemini-cli/{plugin.json, index.ts}
│   │   ├── opencode/{plugin.json, index.ts}
│   │   ├── openclaw/{plugin.json, index.ts}
│   │   └── hermes/{plugin.json, index.ts}
│   └── App.tsx                  ← 修改
└── package.json
```

### Rust 层架构

**命令层 (`commands/tools.rs`):**
```rust
#[tauri::command] fn detect_environment() -> EnvReport
#[tauri::command] fn install_tool(tool_id, root_path) -> Result<()>
#[tauri::command] fn uninstall_tool(tool_id) -> Result<()>
#[tauri::command] fn get_installed_tools() -> Vec<InstalledTool>
#[tauri::command] fn check_tool_updates() -> Vec<ToolUpdateInfo>
```

**服务层 (`services/installer.rs`):**
- `InstallerService` — 安装引擎，管理下载/解压/执行/进度推送
- `PathManager` — Windows PATH 注册、shim 创建和清理

**插件接口 (Rust trait):**
```rust
trait ToolPlugin {
    fn metadata() -> ToolMeta;
    fn detect() -> DetectResult;
    fn install(target_dir: &Path, progress: Sender<Progress>) -> Result<()>;
    fn uninstall(target_dir: &Path) -> Result<()>;
    fn get_prerequisites() -> Vec<Prerequisite>;
}
```

### 数据库

新增 `installed_tools` 表（schema v11）：

```sql
CREATE TABLE installed_tools (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT,
    install_path TEXT NOT NULL,
    install_root TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'not_installed',
    installed_at INTEGER,
    updated_at INTEGER
);
```

迁移策略：在 `apply_schema_migrations` 中添加 v10→v11 分支，遵循 CC Switch 现有的 migration 模式（savepoint、幂等检查、错误回滚）。

### 前端架构

**View 扩展：**
```tsx
type View = "providers" | "settings" | ... | "wizard" | "manager";
```

**首次使用判断：** 检查 `installed_tools` 表是否为空 + localStorage flag

**数据流：**
- `@tanstack/react-query` 管理服务端状态
- Tauri `invoke` 调用 Rust commands
- Tauri `listen` 监听安装进度事件
- `sonner` toast 通知安装完成/失败

## 网络策略

- 启动时检测 `github.com` / `registry.npmjs.org` 连通性
- 不通时展示外部教程链接（GitHub Wiki / 知乎文章），**不内置任何代理或镜像逻辑**
- 所有下载使用 reqwest 直连，无 fallback 机制

## 安装路径方案

- 用户配置统一根目录（如 `D:\AITools`）
- 每个工具装到子目录：`<root>/<tool-id>/`
- Manager 自动管理 PATH 注册和 shim 创建（`<root>/bin/` 下创建 `.cmd` shim）
- Windows 注册表 `HKEY_CURRENT_USER\Environment\PATH` 追加 `<root>\bin`

## 变现路径

| 优先级 | 方式 | 实现 |
|--------|------|------|
| 1 | 中转站推广佣金 | 复用 CC Switch Provider 预设体系，内置合作中转站预设并置顶推荐 |
| 2 | 高级功能付费 | Pro 功能（云同步、团队管理）通过 feature flag 隔离 |
| 3 | 企业版 | 商业合同，与开源版代码隔离 |
| 4 | 流量/影响力 | MIT 许可证，不限制 |

## 不做什么

- 不内置代理/镜像/网络修复逻辑
- 不处理用户 API key 获取（引导去中转站注册）
- 不做配置导入导出（CC Switch 已有）
- 不做 Provider 切换/故障转移（CC Switch 已有）
- 不做 Token 用量统计（CC Switch 已有）
- 首次版本不做 IDE 和 Ollama 插件（保留扩展点）

## 风险点

1. **上游 CC Switch 更新合并冲突** — 保持改动最小化，仅增量添加文件和表，不修改已有核心逻辑
2. **Windows PATH 操作权限** — 需要管理员权限的场景做好提示
3. **npm/GitHub 下载稳定性** — 不内置镜像，依赖用户网络环境
4. **各工具安装方式差异大** — 插件 trait 设计足够灵活，必要时允许插件执行任意命令
