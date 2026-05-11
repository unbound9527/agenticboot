# AgenticBoot v1 实现计划

> **注意：** 计划中所有代码均为伪代码/逻辑描述，不包含真实可执行代码。

**目标：** Fork CC Switch 并添加 AI 编程工具的一键安装/卸载管理能力（软件管家式的向导 + 管理页），支持依赖自动解析安装。

**架构：** 在 CC Switch 现有 Tauri 2 + Rust + React 技术栈上增量添加。新增 Rust 层：`commands/tools.rs` → `services/installer.rs` → 插件 trait 实现。新增数据库表 `installed_tools`（schema v11）。新增前端视图：`wizard` 和 `manager`。插件 trait 定义工具生命周期；DependencyResolver 通过拓扑排序构建安装计划。

**技术栈：** Tauri 2, Rust (rusqlite, reqwest, tokio, winreg), React 18 + TypeScript, Tailwind + shadcn/ui, @tanstack/react-query

---

### 任务 1：Fork CC Switch 并初始化 AgenticBoot 仓库

**涉及文件：**
- 创建：全部项目文件（来自 CC Switch fork）
- 修改：`README.md`, `package.json`（更新名称/元数据）

- [ ] **步骤 1：Clone CC Switch 到当前工作目录**

将 CC Switch 代码库复制到 agenticboot 工作目录，替换现有的占位文件（README.md、package.json）。

```
# 浅克隆 CC Switch
git clone https://github.com/farion1231/cc-switch.git --depth 1 <临时目录>
# 将所有文件复制到工作目录，保留 agenticboot 已有的 .git 目录
```

- [ ] **步骤 2：更新 package.json 元数据**

将 `name` 从 `"cc-switch"` 改为 `"agenticboot"`，更新 `description`、`author`、`repository`、`homepage`、`bugs` 指向 `unbound9527/agenticboot`。

- [ ] **步骤 3：更新 Cargo.toml 元数据**

将 `name` 从 `"cc-switch"` 改为 `"agenticboot"`，更新 `description`、`authors`、`repository`。

- [ ] **步骤 4：更新 tauri.conf.json**

将 `productName` 从 `"CC Switch"` 改为 `"AgenticBoot"`，`identifier` 改为 `com.unbound9527.agenticboot`。

- [ ] **步骤 5：更新 README.md**

替换为当前 agenticboot 仓库已有的中英双语 README 内容。

- [ ] **步骤 6：提交**

```
git add -A
git commit -m "feat: fork CC Switch v3.14.1 as AgenticBoot base"
```

---

### 任务 2：数据库 Schema 迁移（v10 → v11）

**涉及文件：**
- 修改：`src-tauri/src/database/schema.rs` — 添加 `installed_tools` 表和 v10→v11 迁移逻辑

- [ ] **步骤 1：在 `create_tables_on_conn` 中添加 `installed_tools` 建表**

在已有的 `skill_repos` 建表语句之后添加新表：

```
// 伪代码 - CREATE TABLE IF NOT EXISTS installed_tools (
//   id TEXT PRIMARY KEY,              -- 如 "claude-code", "nodejs"
//   name TEXT NOT NULL,               -- 显示名称
//   version TEXT,                     -- 安装后检测到的版本
//   install_path TEXT NOT NULL,       -- 绝对路径
//   install_root TEXT NOT NULL,       -- 用户配置的安装根目录
//   category TEXT NOT NULL DEFAULT 'tool',  -- 'tool' | 'dependency'
//   status TEXT NOT NULL DEFAULT 'not_installed',
//   installed_at INTEGER,
//   updated_at INTEGER
// )
```

- [ ] **步骤 2：在 `apply_schema_migrations_on_conn` 中添加 v10→v11 迁移分支**

在 while 循环中添加新的 match 分支：

```
// 伪代码 - 在 apply_schema_migrations_on_conn 内部:
//   10 => {
//     记录日志：开始迁移
//     创建 installed_tools 表（如果不存在）
//     set_user_version(conn, 11)
//   }
```

遵循 CC Switch 的迁移模式：savepoint → 执行 → 成功则 release / 失败则 rollback。

- [ ] **步骤 3：更新 SCHEMA_VERSION 常量**

在 `database/mod.rs` 中，将 `SCHEMA_VERSION` 从 10 改为 11。

- [ ] **步骤 4：添加 installed_tools 的 DAO 查询方法**

新建 `src-tauri/src/database/dao/tools.rs`：

```
// 伪代码 - 在 Database 上实现的方法：
//   get_installed_tools() -> Vec<InstalledToolRecord>
//     SELECT * FROM installed_tools ORDER BY category, name
//
//   get_installed_tool(id: &str) -> Option<InstalledToolRecord>
//     SELECT * FROM installed_tools WHERE id = ?
//
//   upsert_installed_tool(record: &InstalledToolRecord)
//     INSERT OR REPLACE INTO installed_tools (...) VALUES (...)
//
//   update_tool_status(id: &str, status: &str)
//     UPDATE installed_tools SET status = ?, updated_at = ? WHERE id = ?
//
//   delete_installed_tool(id: &str)
//     DELETE FROM installed_tools WHERE id = ?
//
//   has_any_installed_tools() -> bool
//     SELECT COUNT(*) > 0 FROM installed_tools
```

在 `database/dao/mod.rs` 中注册 `tools` 子模块。

- [ ] **步骤 5：提交**

---

### 任务 3：核心数据类型

**涉及文件：**
- 创建：`src-tauri/src/tool_types.rs` — 所有共享数据类型

- [ ] **步骤 1：定义核心类型**

```
// 伪代码：

// NetworkStatus（网络状态）
//   github_reachable: bool        // github.com 是否可达
//   npm_reachable: bool           // registry.npmjs.org 是否可达
//   error_message: Option<String>

// ToolMeta（工具元信息）
//   id: String                    // 如 "claude-code"
//   name: String                  // 如 "Claude Code"
//   description: String           // 描述
//   icon: String                  // 图标标识
//   category: String              // 分类："ai-cli" | "ai-ide" | "local-model" | "dependency"

// DetectResult（检测结果）
//   installed: bool               // 是否已安装
//   version: Option<String>       // 已安装版本
//   install_path: Option<String>  // 安装路径

// ToolDependency（工具依赖）
//   tool_id: String               // 依赖的工具 ID，如 "nodejs"
//   min_version: Option<String>   // 最低版本要求，如 ">= 18.0.0"

// InstallProgress（安装进度）
//   tool_id: String
//   tool_name: String
//   phase: String                 // "downloading" | "extracting" | "installing" | "complete" | "error"
//   percent: u8                   // 0-100
//   message: String

// InstallPlan（安装计划）
//   steps: Vec<InstallStep>

// InstallStep（安装步骤）
//   tool_id: String
//   tool_name: String
//   category: String
//   reason: String                // 原因："selected"（用户选择）| "dependency_of(xxx)"（依赖）
//   is_installed: bool            // 是否已安装（可跳过）

// InstalledTool（已安装工具记录）
//   id, name, version, install_path, install_root, category, status, installed_at, updated_at

// 所有类型通过 serde 实现 Serialize/Deserialize（前端 camelCase，Rust 侧 snake_case）
// 内部使用实现 Clone, Debug
```

- [ ] **步骤 2：在 lib.rs 中注册模块**

添加 `mod tool_types;` 到 `lib.rs`。

- [ ] **步骤 3：提交**

---

### 任务 4：ToolPlugin Trait 和插件注册表

**涉及文件：**
- 创建：`src-tauri/src/plugin.rs` — trait 定义和注册表

- [ ] **步骤 1：定义 ToolPlugin trait**

```
// 伪代码：

// trait ToolPlugin: Send + Sync {
//     // 返回工具元信息
//     fn metadata() -> ToolMeta;
//
//     // 检测系统上是否已安装该工具
//     fn detect() -> DetectResult;
//
//     // 安装工具到指定目录
//     // target_dir 为工具专属子目录（如 <root>/claude-code/）
//     // progress 用于上报安装进度
//     fn install(target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String>;
//
//     // 从指定目录卸载工具
//     fn uninstall(target_dir: &Path) -> Result<(), String>;
//
//     // 返回该工具依赖的其他工具列表
//     fn get_dependencies() -> Vec<ToolDependency>;
// }
```

- [ ] **步骤 2：构建插件注册表**

```
// 伪代码：

// fn get_all_plugins() -> Vec<Box<dyn ToolPlugin>>:
//   返回包含所有可用插件的列表：
//     // 依赖项
//     NodeJsPlugin, GitPlugin,
//     // CLI 版
//     ClaudeCodeCliPlugin, CodexCliPlugin, GeminiCliPlugin,
//     OpenCodeCliPlugin, OpenClawPlugin, HermesPlugin,
//     // 桌面版
//     ClaudeCodeDesktopPlugin, CodexDesktopPlugin, OpenCodeDesktopPlugin

// fn get_plugin_by_id(id: &str) -> Option<Box<dyn ToolPlugin>>:
//   从 get_all_plugins() 中按 id 查找对应插件
```

- [ ] **步骤 3：在 lib.rs 中注册模块**

添加 `mod plugin;` 到 `lib.rs`。

- [ ] **步骤 4：提交**

---

### 任务 5：依赖解析器

**涉及文件：**
- 创建：`src-tauri/src/services/installer/dependency_resolver.rs`

- [ ] **步骤 1：实现依赖解析逻辑**

```
// 伪代码：

// struct DependencyResolver;

// fn resolve(tool_ids: &[String]) -> Result<InstallPlan, String>:
//
//   // 第一步：收集所有被请求的工具及其传递依赖
//   创建 HashSet<String> all_ids
//   创建 VecDeque<String> queue，初始值为 tool_ids
//   while 从 queue 中弹出 id:
//     if id 不在 all_ids 中:
//       将 id 加入 all_ids
//       if 存在对应插件:
//         for 该插件的每个依赖 dep:
//           将 dep.tool_id 加入 queue
//
//   // 第二步：构建依赖图（邻接表）
//   //   A → B 表示"A 依赖于 B"，即 B 必须先于 A 安装
//   创建 HashMap<String, Vec<String>> graph
//   for all_ids 中的每个 id:
//     graph[id] = 该插件声明的依赖 ID 列表
//
//   // 第三步：拓扑排序（Kahn 算法）
//   //   计算每个节点的入度
//   //   将入度为 0 的节点加入队列
//   //   逐个处理队列，减少后继节点的入度
//   //   如果存在循环依赖 → 返回错误
//   let sorted = 拓扑排序结果
//
//   // 第四步：构建安装计划
//   for sorted 中的每个 id:
//     - 调用 plugin.detect() 检查是否已安装
//     - 如果已安装且版本满足 → step.is_installed = true
//     - 如果未安装或版本过低 → step.is_installed = false
//     - step.reason = 如果 id 在原始 tool_ids 中 → "selected"
//                    否则 → "dependency_of(父工具名称)"
//
//   返回 InstallPlan { steps }
```

- [ ] **步骤 2：提交**

---

### 任务 6：PATH 管理器（Windows）

**涉及文件：**
- 创建：`src-tauri/src/services/installer/path_manager.rs`

- [ ] **步骤 1：实现 PATH 和 shim 管理**

```
// 伪代码：

// struct PathManager { root_dir: PathBuf }
//   root_dir 是用户的统一安装根目录（如 D:\AgenticTools）

// fn new(root_dir: &Path) -> Self

// fn ensure_bin_dir() -> Result<PathBuf, String>:
//   如果不存在则创建 <root>/bin/ 目录
//   返回该路径

// fn register_in_path() -> Result<(), String>:
//   // Windows 下：将 <root>\bin 添加到 HKEY_CURRENT_USER\Environment\PATH
//   // 从注册表读取当前 PATH
//   // 如果 <root>\bin 不在 PATH 中，则追加
//   // 写回注册表
//   // 广播 WM_SETTINGCHANGE 通知系统
//   // 使用 winreg crate（CC Switch 已依赖它做环境检查）

// fn unregister_from_path() -> Result<(), String>:
//   // 从注册表 PATH 中移除 <root>\bin
//   // 仅当 installed_tools 表中无任何已安装工具时才移除

// fn create_shim(tool_id: &str, executable_name: &str) -> Result<(), String>:
//   // 在 <root>/bin/ 下创建 <executable_name>.cmd 文件
//   // 内容：@echo off\n"<工具安装目录>\<可执行文件>.exe" %*
//   // 这样用户可以从任何终端调用该工具

// fn remove_shim(executable_name: &str) -> Result<(), String>:
//   // 删除 <root>/bin/<executable_name>.cmd

// fn get_tool_install_dir(tool_id: &str) -> PathBuf:
//   // 返回 <root>/<tool_id>/
```

- [ ] **步骤 2：提交**

---

### 任务 7：安装引擎服务

**涉及文件：**
- 创建：`src-tauri/src/services/installer/mod.rs`

- [ ] **步骤 1：实现安装引擎**

```
// 伪代码：

// struct InstallerService {
//     root_path: PathBuf,       // 用户配置的安装根目录
//     path_manager: PathManager,
// }

// fn new(root_path: &Path) -> Self

// ---- 网络检测 ----
// fn check_network() -> NetworkStatus:
//   尝试 HTTPS 连接 github.com（reqwest::get，5 秒超时）
//   尝试 HTTPS 连接 registry.npmjs.org（5 秒超时）
//   返回 NetworkStatus（各站点的可达性）

// ---- 安装计划解析 ----
// fn resolve_install_plan(tool_ids: Vec<String>) -> Result<InstallPlan, String>:
//   委托给 DependencyResolver::resolve(&tool_ids)
//   返回安装计划给前端展示

// ---- 执行安装计划 ----
// fn execute_install_plan(plan: &InstallPlan) -> Result<(), String>:
//   for plan.steps 中的每个 step:
//
//     // 跳过已安装的步骤
//     if step.is_installed:
//       推送 Tauri 事件：phase="skipped", message="已安装，跳过"
//       continue
//
//     // 开始安装
//     推送事件：phase="starting", percent=0
//
//     // 获取对应插件
//     plugin = get_plugin_by_id(step.tool_id)
//     target_dir = <root>/<step.tool_id>/
//
//     // 创建进度通道（tokio::mpsc::channel）
//     // 在独立线程中执行：plugin.install(&target_dir, progress_sender)
//     // 监听进度通道，每次收到进度更新时通过 app_handle 推送 Tauri 事件
//     // 等待安装完成
//
//     // 成功时：
//     if 成功:
//       调用 plugin.detect() 获取已安装版本
//       通过 path_manager 创建 shim
//       在 installed_tools 表中 upsert 记录（status="installed"）
//       推送事件：phase="complete"
//
//     // 失败时：
//     if 失败:
//       在 installed_tools 表中 upsert 记录（status="error"）
//       推送事件：phase="error", message=错误信息
//       return 错误

// ---- 卸载工具 ----
// fn uninstall_tool(tool_id: &str) -> Result<(), String>:
//   从 installed_tools 表查找工具
//   如果未找到 → 返回错误
//   如果 category == "dependency":
//     检查是否有其他已安装工具依赖此依赖项
//     如果有 → 允许卸载但前端需弹出警告
//   获取对应插件
//   调用 plugin.uninstall(&install_path)
//   通过 path_manager 移除 shim
//   删除 install_path 目录
//   从 installed_tools 表删除记录

// ---- 查询已安装工具 ----
// fn get_installed_tools() -> Result<Vec<InstalledTool>, String>:
//   从 installed_tools 表查询所有行并返回

// ---- 检查更新 ----
// fn check_tool_updates() -> Result<Vec<ToolUpdateInfo>, String>:
//   for 每个 category=="tool" 的已安装工具:
//     通过插件检测可安装的最新版本
//     与已安装版本对比
//     如果有新版本 → 加入更新列表
//   返回更新列表

// Tauri 事件推送：
//   emit "install-progress" 携带 InstallProgress 数据
//   emit "install-complete" 携带 tool_id
//   emit "install-error" 携带 { tool_id, error }
```

- [ ] **步骤 2：注册 installer 服务模块**

创建 `src-tauri/src/services/installer/mod.rs`，重新导出各子模块。在 `src-tauri/src/services/mod.rs` 中添加 `pub mod installer;`。

- [ ] **步骤 3：提交**

---

### 任务 8：Tauri 命令层

**涉及文件：**
- 创建：`src-tauri/src/commands/tools.rs`

- [ ] **步骤 1：实现 Tauri commands**

```
// 伪代码：

// #[tauri::command]
// fn check_network() -> Result<NetworkStatus, String>:
//   直接调用 InstallerService::check_network()

// #[tauri::command]
// fn resolve_install_plan(tool_ids: Vec<String>) -> Result<InstallPlan, String>:
//   直接调用 DependencyResolver::resolve(&tool_ids)

// #[tauri::command]
// async fn execute_install_plan(
//     root_path: String,
//     app_handle: tauri::AppHandle,
// ) -> Result<(), String>:
//   创建 InstallerService 实例
//   确保 bin 目录存在并注册 PATH
//   执行安装计划（进度事件通过 app_handle 推送）

// #[tauri::command]
// fn uninstall_tool(tool_id: String, root_path: String) -> Result<(), String>:
//   创建 InstallerService 实例
//   调用 service.uninstall_tool(&tool_id)

// #[tauri::command]
// fn get_installed_tools(state: State<AppState>) -> Result<Vec<InstalledTool>, String>:
//   通过 AppState 中的 db 查询 installed_tools 表

// #[tauri::command]
// fn has_any_installed_tools(state: State<AppState>) -> Result<bool, String>:
//   前端用于判断显示向导页还是管家页

// #[tauri::command]
// fn check_tool_updates(state: State<AppState>) -> Result<Vec<ToolUpdateInfo>, String>:
//   调用 InstallerService::check_tool_updates(&state.db)
```

- [ ] **步骤 2：在 commands/mod.rs 中注册**

添加 `mod tools;` 和 `pub use tools::*;`。

- [ ] **步骤 3：在 lib.rs 的 tauri::Builder 中注册**

在 `run()` 函数的 `.invoke_handler(tauri::generate_handler![...])` 中添加新的命令。

- [ ] **步骤 4：提交**

---

### 任务 9：依赖项插件（Node.js、Git）

**涉及文件：**
- 创建：`src-tauri/src/plugins/nodejs.rs`
- 创建：`src-tauri/src/plugins/git.rs`
- 创建：`src-tauri/src/plugins/mod.rs`

- [ ] **步骤 1：实现 Node.js 插件**

```
// 伪代码 - NodeJsPlugin 实现 ToolPlugin：

// fn metadata() -> ToolMeta:
//   id: "nodejs", name: "Node.js", category: "dependency"

// fn detect() -> DetectResult:
//   执行：node --version
//   成功 → 解析版本，installed=true
//   同时检查：where node → 判断是否在托管路径中
//   未找到 → installed=false

// fn get_dependencies() -> Vec<ToolDependency>:
//   Node.js 无依赖，返回空数组

// fn install(target_dir: &Path, progress: Sender<InstallProgress>):
//   下载 Node.js 官方安装器：
//     URL: https://nodejs.org/dist/v{latest_lts}/node-v{latest_lts}-x64.msi
//     下载到临时文件，上报下载进度百分比
//   执行：msiexec /i <msi路径> /qn INSTALLDIR=<target_dir>
//   等待安装完成
//   执行 <target_dir>/node --version 验证
//   上报完成

// fn uninstall(target_dir: &Path) -> Result<(), String>:
//   执行：msiexec /x <产品代码> /qn
//   或直接删除目录（便携式安装的情况）
```

- [ ] **步骤 2：实现 Git 插件**

```
// 伪代码 - GitPlugin 实现 ToolPlugin：

// fn metadata() -> ToolMeta:
//   id: "git", name: "Git", category: "dependency"

// fn detect() -> DetectResult:
//   执行：git --version
//   成功 → installed=true，解析版本

// fn get_dependencies() -> Vec<ToolDependency>:
//   空数组

// fn install(target_dir: &Path, progress: Sender<InstallProgress>):
//   下载 Git for Windows：
//     URL: https://github.com/git-for-windows/git/releases/download/v{version}.windows.1/Git-{version}-64-bit.exe
//     下载到临时文件
//   执行：<installer.exe> /VERYSILENT /NORESTART /DIR=<target_dir>
//   等待安装完成并验证

// fn uninstall(target_dir: &Path) -> Result<(), String>:
//   在 target_dir 中查找卸载程序静默执行
//   或直接删除目录
```

- [ ] **步骤 3：创建 plugins 模块并注册**

创建 `src-tauri/src/plugins/mod.rs` 重新导出所有插件。在 `lib.rs` 中添加 `mod plugins;`。更新 `plugin.rs` 中的 `get_all_plugins()` 注册新插件。

- [ ] **步骤 4：提交**

---

### 任务 10：AI 工具插件 — CLI 版（Claude Code CLI、Codex CLI、Gemini CLI）

**涉及文件：**
- 创建：`src-tauri/src/plugins/claude_code_cli.rs`
- 创建：`src-tauri/src/plugins/codex_cli.rs`
- 创建：`src-tauri/src/plugins/gemini_cli.rs`

- [ ] **步骤 1：实现 Claude Code CLI 插件**

```
// 伪代码 - ClaudeCodeCliPlugin 实现 ToolPlugin：

// fn metadata() →
//   id: "claude-code-cli", name: "Claude Code (CLI)", category: "ai-cli"

// fn detect() → 执行 claude --version

// fn get_dependencies() → [依赖 "nodejs" >= 18]
//   CLI 版通过 npm 安装，需要 Node.js

// fn install(target_dir, progress):
//   步骤：
//     1. 设置 npm prefix 为 target_dir
//     2. 执行：npm install -g @anthropic-ai/claude-code --prefix <target_dir>
//     3. 验证：<target_dir>/bin/claude --version
//     4. 创建 shim：<root>/bin/claude.cmd
//   进度上报："installing" → "configuring" → "complete"

// fn uninstall(target_dir):
//   执行：npm uninstall -g @anthropic-ai/claude-code --prefix <target_dir>
//   或移除 node_modules 中的对应包
//   移除 shim
```

- [ ] **步骤 2：实现 Codex CLI 插件**

```
// 伪代码 - CodexCliPlugin 实现 ToolPlugin：

// fn metadata() → id: "codex-cli", name: "Codex (CLI)", category: "ai-cli"
// fn detect() → 执行 codex --version
// fn get_dependencies() → [依赖 "nodejs" >= 18]
// fn install(target_dir, progress):
//   npm install -g @anthropic-ai/codex --prefix <target_dir>
//   创建 <root>/bin/codex.cmd shim
// fn uninstall(target_dir): npm uninstall 或目录清理
```

- [ ] **步骤 3：实现 Gemini CLI 插件**

```
// 伪代码 - GeminiCliPlugin 实现 ToolPlugin：

// fn metadata() → id: "gemini-cli", name: "Gemini CLI", category: "ai-cli"
// fn detect() → 执行 gemini --version
// fn get_dependencies() → [依赖 "nodejs" >= 18]
// fn install(target_dir, progress):
//   npm install -g @anthropic-ai/gemini-cli --prefix <target_dir>
//   创建 shim
// fn uninstall(target_dir): npm uninstall 或目录清理
```

- [ ] **步骤 4：在插件注册表中注册**

- [ ] **步骤 5：提交**

---

### 任务 10b：AI 工具插件 — 桌面版（Claude Code Desktop、Codex Desktop、OpenCode Desktop）

**涉及文件：**
- 创建：`src-tauri/src/plugins/claude_code_desktop.rs`
- 创建：`src-tauri/src/plugins/codex_desktop.rs`
- 创建：`src-tauri/src/plugins/opencode_desktop.rs`

- [ ] **步骤 1：实现 Claude Code Desktop 插件**

```
// 伪代码 - ClaudeCodeDesktopPlugin 实现 ToolPlugin：

// fn metadata() →
//   id: "claude-code-desktop", name: "Claude Code (桌面版)", category: "ai-cli"

// fn detect() → 检查桌面版安装路径是否存在
//   默认路径：%LOCALAPPDATA%/Programs/Claude Code/

// fn get_dependencies() → 空数组
//   桌面版自带运行时，不依赖 Node.js

// fn install(target_dir, progress):
//   安装 Claude Code 桌面版（Tauri 桌面应用）
//   步骤：
//     1. 从 GitHub Releases 或 Anthropic 官方下载 .exe 安装器
//     2. 执行静默安装：<installer.exe> /S /D=<target_dir>
//     3. 验证：检查 <target_dir>/Claude Code.exe 是否存在
//     4. 创建 shim：<root>/bin/claude-desktop.cmd → 启动桌面应用
//   进度上报："downloading" → "installing" → "configuring" → "complete"

// fn uninstall(target_dir):
//   执行桌面版卸载程序：<target_dir>/uninstall.exe /S
//   或直接删除目录
//   移除 shim
```

- [ ] **步骤 2：实现 Codex Desktop 插件**

```
// 伪代码 - CodexDesktopPlugin 实现 ToolPlugin：

// fn metadata() →
//   id: "codex-desktop", name: "Codex (桌面版)", category: "ai-cli"

// fn detect() → 检查桌面版安装路径

// fn get_dependencies() → 空数组
//   桌面版自带运行时

// fn install(target_dir, progress):
//   步骤：
//     1. 从 GitHub Releases 或 winget 获取 Codex 桌面版安装包
//     2. 静默安装到 target_dir
//     3. 验证可执行文件
//     4. 创建 shim：<root>/bin/codex-desktop.cmd
//   进度上报："downloading" → "installing" → "complete"

// fn uninstall(target_dir):
//   执行卸载程序或删除目录，移除 shim
```

- [ ] **步骤 3：实现 OpenCode Desktop 插件**

```
// 伪代码 - OpenCodeDesktopPlugin 实现 ToolPlugin：

// fn metadata() →
//   id: "opencode-desktop", name: "OpenCode (桌面版)", category: "ai-cli"

// fn detect() → 检查桌面版安装路径

// fn get_dependencies() → 空数组
//   桌面版自带运行时

// fn install(target_dir, progress):
//   步骤：
//     1. 从 GitHub Releases 获取最新 OpenCode 桌面版安装包
//     2. 下载到临时文件，上报下载进度
//     3. 解压/安装到 target_dir
//     4. 验证可执行文件
//     5. 创建 shim：<root>/bin/opencode-desktop.cmd
//   进度上报："downloading" → "extracting" → "installing" → "complete"

// fn uninstall(target_dir):
//   删除 target_dir 下文件，移除 shim
```

- [ ] **步骤 4：在插件注册表中注册全部三个**

- [ ] **步骤 5：提交**

---

### 任务 11：AI 工具插件（OpenCode CLI、OpenClaw、Hermes Web UI）

**涉及文件：**
- 创建：`src-tauri/src/plugins/opencode_cli.rs`
- 创建：`src-tauri/src/plugins/openclaw.rs`
- 创建：`src-tauri/src/plugins/hermes.rs`

- [ ] **步骤 1：实现 OpenCode CLI 插件**

```
// 伪代码 - OpenCodeCliPlugin 实现 ToolPlugin：

// fn metadata() → id: "opencode-cli", name: "OpenCode (CLI)", category: "ai-cli"
// fn detect() → 执行 opencode --version
// fn get_dependencies() → [依赖 "nodejs" >= 18]
// fn install(target_dir, progress):
//   npm install -g opencode --prefix <target_dir>
//   创建 shim：<root>/bin/opencode.cmd
// fn uninstall(target_dir): npm uninstall 或目录清理
```

- [ ] **步骤 2：实现 OpenClaw 插件**

```
// 伪代码 - OpenClawPlugin 实现 ToolPlugin：
//   id: "openclaw", name: "OpenClaw", category: "ai-cli"
//   detect() → 执行 openclaw --version
//   get_dependencies() → [依赖 "nodejs" >= 18]
//   install() → 从 GitHub Release 下载 Windows x64 二进制文件
//               解压到 target_dir，创建 shim
//   uninstall() → 删除二进制文件和目录
```

- [ ] **步骤 3：实现 Hermes Web UI 插件**

```
// 伪代码 - HermesPlugin 实现 ToolPlugin：

// fn metadata() → id: "hermes", name: "Hermes (Web UI)", category: "ai-cli"

// fn detect() → 执行 hermes --version
//   同时检查 Web UI 服务是否可访问（默认端口 localhost:xxxx）

// fn get_dependencies() → [依赖 "nodejs" >= 18]

// fn install(target_dir, progress):
//   Hermes 通过 npm 安装，侧重 Web UI 模式
//   步骤：
//     1. 设置 npm prefix 为 target_dir
//     2. 执行：npm install -g hermes --prefix <target_dir>
//     3. 验证：<target_dir>/bin/hermes --version
//     4. 创建两个 shim：
//        <root>/bin/hermes.cmd → 标准 CLI 入口
//        <root>/bin/hermes-webui.cmd → 启动 Web UI（内容：hermes --webui）
//        启动后用户浏览器打开 Web UI 交互
//   进度上报："installing" → "configuring" → "complete"

// fn uninstall(target_dir):
//   执行：npm uninstall -g hermes --prefix <target_dir>
//   移除 hermes.cmd 和 hermes-webui.cmd 两个 shim
//   或直接删除 node_modules 中的对应包
```

- [ ] **步骤 4：在插件注册表中注册全部三个**

- [ ] **步骤 5：提交**

---

### 任务 12：状态和连接层

**涉及文件：**
- 修改：`src-tauri/src/store.rs` — 添加安装相关状态
- 修改：`src-tauri/src/lib.rs` — 初始化服务

- [ ] **步骤 1：将安装根目录路径作为设置存储**

复用 CC Switch 已有的 `settings` 键值表：

```
// 伪代码 - 在 Database 上添加方法：
//   fn get_install_root() -> Option<String>:
//     SELECT value FROM settings WHERE key = 'install_root'
//   fn set_install_root(path: &str):
//     INSERT OR REPLACE INTO settings (key, value) VALUES ('install_root', ?)
```

不需要修改 `AppState` 结构体 —— 安装根目录在每次操作时从 DB settings 中读取，不常驻内存。

- [ ] **步骤 2：在 lib.rs setup 中注册所有命令**

在 `run()` 函数中，将所有 tools 命令添加到 `invoke_handler`。InstallerService 在每次操作时根据 root_path 创建即可。

- [ ] **步骤 3：提交**

---

### 任务 13：前端类型定义

**涉及文件：**
- 创建：`src/types/tools.ts`

- [ ] **步骤 1：定义与 Rust 类型对应的 TypeScript 类型**

```
// 伪代码（TypeScript 类型）：

// type NetworkStatus = {
//   githubReachable: boolean
//   npmReachable: boolean
//   errorMessage?: string
// }

// type ToolMeta = {
//   id: string
//   name: string
//   description: string
//   icon: string
//   category: 'ai-cli' | 'ai-ide' | 'local-model' | 'dependency'
// }

// type InstallStep = {
//   toolId: string
//   toolName: string
//   category: string
//   reason: string         // "selected" | "dependency_of(Claude Code)"
//   isInstalled: boolean
// }

// type InstallPlan = {
//   steps: InstallStep[]
// }

// type InstallProgress = {
//   toolId: string
//   toolName: string
//   phase: 'starting' | 'downloading' | 'extracting' | 'installing' | 'configuring' | 'complete' | 'error' | 'skipped'
//   percent: number        // 0-100
//   message: string
// }

// type InstalledTool = {
//   id: string
//   name: string
//   version?: string
//   installPath: string
//   installRoot: string
//   category: 'tool' | 'dependency'
//   status: 'not_installed' | 'installing' | 'installed' | 'error'
//   installedAt?: number
//   updatedAt?: number
// }

// type ToolUpdateInfo = {
//   toolId: string
//   currentVersion: string
//   latestVersion: string
// }
```

- [ ] **步骤 2：提交**

---

### 任务 14：前端 API 层

**涉及文件：**
- 创建：`src/lib/api/tools.ts`

- [ ] **步骤 1：创建 tools API 模块**

```
// 伪代码（API 函数）：

// import { invoke } from '@tauri-apps/api/core'
// import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// export const toolsApi = {
//
//   // ---- 命令调用 ----
//   checkNetwork(): Promise<NetworkStatus>:
//     invoke('check_network')
//
//   resolveInstallPlan(toolIds: string[]): Promise<InstallPlan>:
//     invoke('resolve_install_plan', { toolIds })
//
//   executeInstallPlan(rootPath: string): Promise<void>:
//     invoke('execute_install_plan', { rootPath })
//
//   uninstallTool(toolId: string, rootPath: string): Promise<void>:
//     invoke('uninstall_tool', { toolId, rootPath })
//
//   getInstalledTools(): Promise<InstalledTool[]>:
//     invoke('get_installed_tools')
//
//   hasAnyInstalledTools(): Promise<boolean>:
//     invoke('has_any_installed_tools')
//
//   checkToolUpdates(): Promise<ToolUpdateInfo[]>:
//     invoke('check_tool_updates')
//
//   // ---- 事件监听 ----
//   onInstallProgress(callback): Promise<UnlistenFn>:
//     // 监听 'install-progress' Tauri 事件
//     listen<InstallProgress>('install-progress', callback)
//
//   onInstallComplete(callback): Promise<UnlistenFn>:
//     // 监听 'install-complete' 事件
//     listen<string>('install-complete', callback)
//
//   onInstallError(callback): Promise<UnlistenFn>:
//     // 监听 'install-error' 事件
//     listen('install-error', callback)
// }
```

- [ ] **步骤 2：提交**

---

### 任务 15：React Query Hooks

**涉及文件：**
- 创建：`src/hooks/useTools.ts`
- 创建：`src/hooks/useInstallProgress.ts`

- [ ] **步骤 1：创建 useTools hook**

```
// 伪代码：

// 使用 @tanstack/react-query 的 useQuery / useMutation / useQueryClient

// useInstalledTools():
//   useQuery({ queryKey: ['installed-tools'], queryFn: toolsApi.getInstalledTools })

// useHasInstalledTools():
//   useQuery({ queryKey: ['has-installed-tools'], queryFn: toolsApi.hasAnyInstalledTools })

// useCheckNetwork():
//   useQuery({ queryKey: ['network-status'], queryFn: toolsApi.checkNetwork, retry: false })

// useResolveInstallPlan():
//   useMutation({ mutationFn: (toolIds: string[]) => toolsApi.resolveInstallPlan(toolIds) })

// useExecuteInstallPlan():
//   useMutation({
//     mutationFn: (rootPath: string) => toolsApi.executeInstallPlan(rootPath),
//     onSuccess: () => 刷新 installed-tools 查询缓存
//   })

// useUninstallTool():
//   useMutation({
//     mutationFn: ({ toolId, rootPath }) => toolsApi.uninstallTool(toolId, rootPath),
//     onSuccess: () => 刷新 installed-tools 查询缓存
//   })
```

- [ ] **步骤 2：创建 useInstallProgress hook**

```
// 伪代码：

// useInstallProgress():
//   const [progressMap, setProgressMap] = useState<Map<string, InstallProgress>>(new Map())
//
//   useEffect:
//     监听 toolsApi.onInstallProgress，每收到进度更新：
//       将 progress 存入 progressMap（以 toolId 为 key）
//     返回取消监听的函数
//
//   提供辅助方法：
//     getToolProgress(toolId): 返回该工具的当前进度或 null
//     allComplete: 所有步骤的 phase 均为 complete/skipped/error
//     hasErrors: 任意步骤 phase == "error"
//
//   返回 { progressMap, getToolProgress, allComplete, hasErrors }
```

- [ ] **步骤 3：提交**

---

### 任务 16：网络检测面板组件

**涉及文件：**
- 创建：`src/components/tools/EnvCheckPanel.tsx`

- [ ] **步骤 1：构建网络检测面板**

```
// 伪代码（React 组件）：

// Props: { onNext: () => void }
// 使用 useCheckNetwork() hook

// 三种状态：
//   loading（加载中）:
//     显示加载动画 + "检测网络连接..."
//
//   error（网络异常）:
//     显示警告图标 + "网络连接异常"
//     显示提示："请先解决网络问题再继续安装。"
//     链接到外部网络问题解决指南（GitHub Wiki 或知乎文章 URL）
//     重试按钮
//
//   success（网络正常）:
//     显示勾选图标 + "网络连接正常"
//     自动 1 秒后进入下一步，或显示"下一步"按钮

// 布局：居中卡片，包含图标、状态文字、操作按钮
```

- [ ] **步骤 2：提交**

---

### 任务 17：安装路径配置组件

**涉及文件：**
- 创建：`src/components/tools/PathConfig.tsx`

- [ ] **步骤 1：构建路径配置组件**

```
// 伪代码（React 组件）：

// Props: { onNext: (rootPath: string) => void, onBack: () => void }

// State:
//   rootPath: string（Windows 默认值 "C:\\AgenticTools"）

// UI:
//   - 输入框，标签"安装根目录"
//   - 浏览按钮 → 使用 Tauri dialog 插件选择目录
//   - 预览区域，展示目录树结构：
//       C:\AgenticTools\
//         bin\           ← shim 脚本目录
//         claude-code\
//         codex\
//         gemini-cli\
//         ...
//   - "上一步"和"下一步"按钮
//   - 验证：路径必须存在或可创建，不能是系统目录

// 点击"下一步"→ 调用 onNext(rootPath)
```

- [ ] **步骤 2：提交**

---

### 任务 18：安装进度组件

**涉及文件：**
- 创建：`src/components/tools/InstallProgress.tsx`

- [ ] **步骤 1：构建安装进度展示**

```
// 伪代码（React 组件）：

// Props: {
//   installPlan: InstallPlan,
//   onComplete: () => void,
//   onError: (toolId: string, error: string) => void
// }
// 使用 useInstallProgress() hook

// UI:
//   顶部总进度条：
//     (已完成步数 + 已跳过步数) / 总步数 × 100%
//
//   步骤列表（来自 installPlan.steps），每行显示：
//     - 工具图标/名称
//     - 分类标签（"已选择" | "依赖" | "已安装"）
//     - 状态指示器：
//         pending（等待中）: 灰色圆点
//         active（进行中）: 旋转加载动画 + 进度条 (0-100%)
//         complete（完成）: 绿色勾选
//         skipped（跳过）: 灰色"已安装"标签
//         error（失败）:   红色 X + 错误信息
//     - 阶段文字："下载中..." / "安装中..." / "配置中..." / "完成"
//
//   全部完成时：
//     显示"安装完成！"
//     "进入管理"按钮 → 调用 onComplete()
//
//   有失败项时：
//     显示失败工具列表及错误信息
//     "重试失败项"按钮
```

- [ ] **步骤 2：提交**

---

### 任务 19：向导页

**涉及文件：**
- 创建：`src/pages/Wizard.tsx`

- [ ] **步骤 1：构建首次向导页面**

```
// 伪代码（React 组件）：

// Props: { onComplete: () => void }

// 多步骤向导，顶部显示步骤指示器：
//   第 1 步：网络检测（EnvCheckPanel）
//   第 2 步：安装路径（PathConfig）
//   第 3 步：选择工具（内联工具清单）
//   第 4 步：安装中（InstallProgress）

// State:
//   currentStep: 1 | 2 | 3 | 4
//   rootPath: string = ""
//   selectedTools: string[] = []
//   installPlan: InstallPlan | null = null

// 第 3 步 UI（工具选择）:
//   工具网格布局，每个卡片显示：
//     - 图标
//     - 名称 + 简短描述
//     - 勾选框（默认全选）
//     - "依赖: Node.js" 标签（仅 CLI 版）
//   CLI 版和桌面版分开展示，如：
//     Claude Code (CLI)     — 需要 Node.js
//     Claude Code (桌面版)   — 无需额外依赖
//     Codex (CLI)           — 需要 Node.js
//     Codex (桌面版)         — 无需额外依赖
//     OpenCode (CLI)        — 需要 Node.js
//     OpenCode (桌面版)      — 无需额外依赖
//     Gemini CLI            — 需要 Node.js
//     OpenClaw              — 需要 Node.js
//     Hermes (Web UI)       — 需要 Node.js
//   "全部勾选" / "全部取消" 切换按钮
//   "上一步"和"开始安装"按钮
//
//   点击"开始安装":
//     调用 resolveInstallPlan(selectedTools)
//     展示安装计划（含哪些依赖会被自动安装）
//     用户确认后 → 进入第 4 步
//     调用 executeInstallPlan(rootPath)

// 第 4 步：展示 <InstallProgress plan={installPlan} onComplete={onComplete} />

// 页面布局：居中卡片，max-w-2xl，顶部步骤圆点
```

- [ ] **步骤 2：提交**

---

### 任务 20：管家页

**涉及文件：**
- 创建：`src/pages/Manager.tsx`

- [ ] **步骤 1：构建日常管理页面**

```
// 伪代码（React 组件）：

// 使用 hooks：useInstalledTools(), useUninstallTool(), useCheckToolUpdates()

// UI 布局：
//   标题栏："工具管理" + "检查更新"按钮
//
//   双 Tab 布局（软件管家风格）：
//
//   ┌─ Tab "已安装"（默认显示）:
//   │  已安装工具的卡片/网格列表，每项显示：
//   │    - 工具图标 + 名称 + 版本
//   │    - 安装路径（截断显示）
//   │    - 状态标签："已安装"（绿色）
//   │    - 操作按钮："卸载"按钮、"更新"按钮（如果有新版本）
//   │
//   └─ Tab "未安装":
//      未安装工具的网格列表：
//        - 工具图标 + 名称 + 描述
//        - "安装"按钮
//        点击"安装" → 解析该工具及其依赖的计划 →
//          弹出确认对话框 → 安装 → 刷新列表
//
//   每项工具卡片使用 ToolCard 组件
//
//   卸载流程：
//     点击"卸载" → 弹出确认对话框
//       如果 category=="dependency": 警告"其他工具可能依赖此项"
//       "确认卸载" → 调用 uninstallTool(toolId, rootPath) → 刷新列表
//
//   设置区域（可折叠）：
//     当前安装根目录路径（只读，有"修改"按钮 → 打开 PathConfig）
```

- [ ] **步骤 2：提交**

---

### 任务 21：ToolCard 组件

**涉及文件：**
- 创建：`src/components/tools/ToolCard.tsx`

- [ ] **步骤 1：构建可复用的工具卡片**

```
// 伪代码（React 组件）：

// Props: {
//   tool: InstalledTool | ToolMeta
//   variant: 'installed' | 'available'
//   onInstall?: () => void
//   onUninstall?: () => void
//   onUpdate?: () => void
//   progress?: InstallProgress  // 安装进行中的进度数据
// }

// UI（使用 shadcn/ui Card, Badge, Button, Progress 组件）:
//   卡片布局，分为左、中、右三栏：
//
//   左侧：
//     工具图标（使用 Lucide 图标或工具专用 SVG）
//
//   中间：
//     工具名称（加粗）
//     版本号或描述（小字、灰色）
//     如果正在安装且有 progress：迷你进度条
//
//   右侧：
//     如果 variant=='installed':
//       "已安装 v1.2.3" 标签（绿色）
//       "卸载"按钮（危险操作样式）
//       "更新"按钮（如果有新版本，蓝色边框样式）
//     如果 variant=='available':
//       "安装"按钮（主色）
//     如果 progress 存在且正在安装中:
//       替换按钮区域为进度指示器
```

- [ ] **步骤 2：提交**

---

### 任务 22：App.tsx 集成

**涉及文件：**
- 修改：`src/App.tsx`

- [ ] **步骤 1：向 View 类型添加 wizard 和 manager 视图**

```
// 向 View 类型联合添加:
//   | "wizard"
//   | "manager"

// 向 VALID_VIEWS 数组添加:
//   ..., "wizard", "manager"
```

- [ ] **步骤 2：添加导航入口**

```
// 在侧边栏/工具栏导航中:
//   添加"装机"导航项
//   图标：使用 lucide-react 的 Wrench 或 Package 图标
//   点击行为：
//     检查 hasAnyInstalledTools()
//       如果为 false → setCurrentView("wizard")
//       如果为 true → setCurrentView("manager")
```

- [ ] **步骤 3：添加视图渲染逻辑**

```
// 在主渲染 switch/case 中:
//   case "wizard":
//     return <Wizard onComplete={() => setCurrentView("manager")} />
//   case "manager":
//     return <Manager onInstallMore={() => setCurrentView("wizard")} />
```

- [ ] **步骤 4：处理首次启动逻辑**

```
// 应用 mount 时:
//   检查 hasAnyInstalledTools()
//   如果为 false 且 localStorage 中没有 "wizard-seen" 标记:
//     自动导航到向导页 setCurrentView("wizard")
//   向导完成后设置 localStorage 标记
```

- [ ] **步骤 5：提交**

---

### 任务 23：i18n 国际化字符串

**涉及文件：**
- 修改：`src/i18n/` — 添加工具管理 UI 的翻译键

- [ ] **步骤 1：添加中文（zh）翻译**

```
// 向中文翻译 JSON 添加：

// tools.title: "工具管理"
// tools.wizard: "装机向导"
// tools.manager: "软件管家"
// tools.install: "安装"
// tools.uninstall: "卸载"
// tools.update: "更新"
// tools.installed: "已安装"
// tools.notInstalled: "未安装"
// tools.installing: "安装中..."
// tools.checkNetwork: "检测网络连接"
// tools.networkOk: "网络连接正常"
// tools.networkError: "网络连接异常，请先解决网络问题"
// tools.networkGuide: "查看网络问题解决指南"
// tools.selectRoot: "选择安装目录"
// tools.selectTools: "选择要安装的工具"
// tools.startInstall: "开始安装"
// tools.installComplete: "安装完成！"
// tools.uninstallConfirm: "确认卸载 {{name}}？"
// tools.dependencyWarning: "{{name}} 是其他工具的依赖项，卸载可能导致相关工具无法使用"
// tools.previous: "上一步"
// tools.next: "下一步"
// tools.retry: "重试"
// tools.all: "全部勾选"
// tools.none: "全部取消"
```

- [ ] **步骤 2：添加英文（en）翻译**

相同的 key，英文值。

- [ ] **步骤 3：提交**

---

### 任务 24：集成测试和冒烟测试

**涉及文件：**
- 创建：`src-tauri/tests/tool_plugins_tests.rs`

- [ ] **步骤 1：编写依赖解析的集成测试**

```
// 伪代码 - Rust 集成测试：

// #[test] test_dependency_resolver_no_deps():
//   选择一个无依赖的工具（如 Node.js）
//   计划应恰好包含 1 个步骤

// #[test] test_dependency_resolver_with_deps():
//   选择 "Claude Code"
//   计划应为 ["nodejs", "claude-code"] 且顺序正确
//   Node.js 的 reason = "dependency_of(Claude Code)"
//   Claude Code 的 reason = "selected"

// #[test] test_dependency_resolver_dedup():
//   同时选择 "Claude Code" 和 "Codex"
//   两者都依赖 Node.js
//   计划中 Node.js 只出现一次
//   顺序：nodejs, 然后 claude-code, 然后 codex（或 codex 然后 claude-code）

// #[test] test_dependency_resolver_cycle_detection():
//   如果插件存在循环依赖（理论上不会，但要测）
//   应返回错误，不应卡死
```

- [ ] **步骤 2：手动冒烟测试**

```
// 手动验证清单：
// 1.  应用启动，无任何已安装工具 → 显示向导页
// 2.  网络检测正常（通过/失败两种状态）
// 3.  路径选择正常工作（浏览对话框、验证）
// 4.  工具选择：勾选/取消勾选工具
// 5.  安装计划展示正确的依赖顺序
// 6.  安装过程中进度事件正确接收和显示
// 7.  安装完成后管家页显示已安装工具
// 8.  卸载流程正常（确认 → 卸载 → 列表更新）
// 9.  全部卸载后 → 再次显示向导页
// 10. 已有的 CC Switch 功能不受影响（Provider、Proxy、Settings）
```

- [ ] **步骤 3：修复冒烟测试发现的 bug**

逐个修复 bug，每个修复单独提交。

- [ ] **步骤 4：最终提交**

---

### 任务 25：CI / 构建验证

**涉及文件：**
- 修改：`.github/workflows/` — 检查 CI 配置是否需要更新

- [ ] **步骤 1：在 Windows 上验证 Tauri 构建**

```
执行：pnpm tauri build（或 cargo build in src-tauri）
预期：编译成功，生成 .msi 安装包
```

- [ ] **步骤 2：按需更新 GitHub workflow**

检查 CC Switch 已有的 CI 工作流是否与新名称/配置兼容。仅当失败才修改 —— fork 应保留已有的 CI 配置。

- [ ] **步骤 3：提交 CI 修复（如果有）**

---

## 实现顺序说明

- 任务 1-12（Rust 后端）与任务 13-22（前端）可以基本并行进行
- 任务 3→5→6→7→8 有顺序依赖：类型 → 解析器 → PATH 管理 → 安装引擎 → 命令
- 任务 9-11 依赖任务 4（trait 定义）
- 任务 14 依赖 13（类型），任务 15 依赖 14（API）
- 任务 16-18（组件）可并行构建
- 任务 19-20（页面）依赖组件
- 任务 22（App.tsx 集成）是最终的前端整合步骤
- 任务 24（测试）和 25（CI）在后端和前端都完成后进行

## 文件清单

**创建（26 个新文件）：**

| 文件 | 用途 |
|------|------|
| `src-tauri/src/tool_types.rs` | 共享数据类型 |
| `src-tauri/src/plugin.rs` | ToolPlugin trait + 注册表 |
| `src-tauri/src/services/installer/mod.rs` | 安装引擎门面 |
| `src-tauri/src/services/installer/dependency_resolver.rs` | 拓扑排序依赖解析 |
| `src-tauri/src/services/installer/path_manager.rs` | PATH + shim 管理 |
| `src-tauri/src/commands/tools.rs` | Tauri 命令处理器 |
| `src-tauri/src/plugins/mod.rs` | 插件模块入口 |
| `src-tauri/src/plugins/nodejs.rs` | Node.js 插件 |
| `src-tauri/src/plugins/git.rs` | Git 插件 |
| `src-tauri/src/plugins/claude_code.rs` | Claude Code 插件 |
| `src-tauri/src/plugins/codex.rs` | Codex 插件 |
| `src-tauri/src/plugins/gemini_cli.rs` | Gemini CLI 插件 |
| `src-tauri/src/plugins/opencode.rs` | OpenCode 插件 |
| `src-tauri/src/plugins/openclaw.rs` | OpenClaw 插件 |
| `src-tauri/src/plugins/hermes.rs` | Hermes 插件 |
| `src-tauri/src/database/dao/tools.rs` | installed_tools 数据库查询 |
| `src/types/tools.ts` | 前端 TypeScript 类型 |
| `src/lib/api/tools.ts` | 前端 API 绑定 |
| `src/hooks/useTools.ts` | React Query hooks |
| `src/hooks/useInstallProgress.ts` | 进度事件 hook |
| `src/components/tools/EnvCheckPanel.tsx` | 网络检测面板 |
| `src/components/tools/PathConfig.tsx` | 路径配置组件 |
| `src/components/tools/InstallProgress.tsx` | 进度展示组件 |
| `src/components/tools/ToolCard.tsx` | 可复用工具卡片 |
| `src/pages/Wizard.tsx` | 首次向导页 |
| `src/pages/Manager.tsx` | 日常管家页 |
| `src-tauri/tests/tool_plugins_tests.rs` | 集成测试 |

**修改（12 个已有文件）：**

| 文件 | 修改内容 |
|------|---------|
| `package.json` | 更新元数据 |
| `src-tauri/Cargo.toml` | 更新元数据 |
| `src-tauri/tauri.conf.json` | 更新产品名/标识符 |
| `README.md` | 替换为 AgenticBoot 内容 |
| `src-tauri/src/lib.rs` | 添加模块声明、注册命令 |
| `src-tauri/src/database/schema.rs` | v11 迁移 |
| `src-tauri/src/database/mod.rs` | SCHEMA_VERSION 加 1 |
| `src-tauri/src/database/dao/mod.rs` | 注册 tools DAO |
| `src-tauri/src/services/mod.rs` | 注册 installer 模块 |
| `src-tauri/src/commands/mod.rs` | 注册 tools 命令 |
| `src/App.tsx` | 添加 wizard/manager 视图 |
| `src/i18n/` | 添加翻译键 |
