# AgenticBoot v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fork CC Switch and add AI coding tool installation/management (app-store-like wizard + manager) with auto-dependency resolution.

**Architecture:** Incremental additions to CC Switch's existing Tauri 2 + Rust + React stack. New Rust layer: `commands/tools.rs` → `services/installer.rs` → plugin trait implementations. New DB table `installed_tools` at schema v11. New frontend views: `wizard` and `manager`. Plugin trait defines tool lifecycle; DependencyResolver builds install plans via topological sort.

**Tech Stack:** Tauri 2, Rust (rusqlite, reqwest, tokio, winreg), React 18 + TypeScript, Tailwind + shadcn/ui, @tanstack/react-query

---

### Task 1: Fork CC Switch and Initialize AgenticBoot Repository

**Files:**
- Create: all project files (from CC Switch fork)
- Modify: `README.md`, `package.json` (update name/metadata)

- [ ] **Step 1: Fork CC Switch and set up as origin**

Clone CC Switch into the agenticboot repo. The existing agenticboot placeholder files (README.md, package.json) will be replaced by the CC Switch codebase.

Approach: Clone CC Switch into a temp location, copy all files into the agenticboot working directory, then customize metadata.

```
# Clone CC Switch (shallow, to save time)
git clone https://github.com/farion1231/cc-switch.git --depth 1 <temp_dir>

# Copy all files into the working directory, preserving .gitignore etc
# but keeping the existing agenticboot .git directory
```

- [ ] **Step 2: Update package.json metadata**

Change `name` from `"cc-switch"` to `"agenticboot"`, update `description`, `author`, `repository`, `homepage`, `bugs` URLs to point to `unbound9527/agenticboot`.

- [ ] **Step 3: Update Cargo.toml metadata**

Change `name` from `"cc-switch"` to `"agenticboot"`, update `description`, `authors`, `repository`.

- [ ] **Step 4: Update tauri.conf.json**

Change `productName` from `"CC Switch"` to `"AgenticBoot"`, update `identifier` to `com.unbound9527.agenticboot`.

- [ ] **Step 5: Update README.md**

Replace with the existing AgenticBoot README content (EN/ZH bilingual, already written in the current agenticboot repo).

- [ ] **Step 6: Commit**

```
git add -A
git commit -m "feat: fork CC Switch v3.14.1 as AgenticBoot base"
```

---

### Task 2: Database Schema Migration (v10 → v11)

**Files:**
- Modify: `src-tauri/src/database/schema.rs` — add `installed_tools` table and v10→v11 migration

- [ ] **Step 1: Add `installed_tools` table to `create_tables_on_conn`**

Add a new table creation block after the existing `skill_repos` table:

```
// Pseudocode - CREATE TABLE IF NOT EXISTS installed_tools (
//   id TEXT PRIMARY KEY,              -- e.g. "claude-code", "nodejs"
//   name TEXT NOT NULL,               -- display name
//   version TEXT,                     -- detected after install
//   install_path TEXT NOT NULL,       -- absolute path
//   install_root TEXT NOT NULL,       -- user's root dir setting
//   category TEXT NOT NULL DEFAULT 'tool',  -- 'tool' | 'dependency'
//   status TEXT NOT NULL DEFAULT 'not_installed',
//   installed_at INTEGER,
//   updated_at INTEGER
// )
```

- [ ] **Step 2: Add v10→v11 migration branch in `apply_schema_migrations_on_conn`**

Add a new match arm in the while loop:

```
// Pseudocode - inside apply_schema_migrations_on_conn:
//   10 => {
//     log migration start
//     create installed_tools table if not exists
//     set_user_version(conn, 11)
//   }
```

Follow CC Switch's migration pattern: savepoint → execute → release on success / rollback on error.

- [ ] **Step 3: Bump SCHEMA_VERSION constant**

In `database/mod.rs`, change `SCHEMA_VERSION` from 10 to 11.

- [ ] **Step 4: Add DAO queries for installed_tools**

New file `src-tauri/src/database/dao/tools.rs`:

```
// Pseudocode - functions on Database impl:
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

Register in `database/dao/mod.rs`.

- [ ] **Step 5: Commit**

```
git add src-tauri/src/database/
git commit -m "feat: add installed_tools table and v10→v11 schema migration"
```

---

### Task 3: Core Data Types

**Files:**
- Create: `src-tauri/src/tool_types.rs` — all shared types

- [ ] **Step 1: Define core types in `tool_types.rs`**

```
// Pseudocode:

// NetworkStatus
//   github_reachable: bool
//   npm_reachable: bool
//   error_message: Option<String>

// ToolMeta
//   id: String           // e.g. "claude-code"
//   name: String          // e.g. "Claude Code"
//   description: String
//   icon: String          // icon identifier
//   category: String      // "ai-cli" | "ai-ide" | "local-model" | "dependency"

// DetectResult
//   installed: bool
//   version: Option<String>
//   install_path: Option<String>

// ToolDependency
//   tool_id: String       // e.g. "nodejs"
//   min_version: Option<String>  // e.g. ">= 18.0.0"

// InstallProgress
//   tool_id: String
//   tool_name: String
//   phase: String         // "downloading" | "extracting" | "installing" | "complete" | "error"
//   percent: u8           // 0-100
//   message: String

// InstallPlan
//   steps: Vec<InstallStep>

// InstallStep
//   tool_id: String
//   tool_name: String
//   category: String
//   reason: String        // "selected" | "dependency_of(tool_name)"
//   is_installed: bool    // already satisfied

// InstalledTool
//   id, name, version, install_path, install_root, category, status, installed_at, updated_at

// Serialize/Deserialize for all types via serde
// Clone, Debug for internal Rust use
```

- [ ] **Step 2: Register the module**

Add `mod tool_types;` to `lib.rs`.

- [ ] **Step 3: Commit**

```
git add src-tauri/src/tool_types.rs src-tauri/src/lib.rs
git commit -m "feat: add core data types for tool management"
```

---

### Task 4: ToolPlugin Trait and Plugin Registry

**Files:**
- Create: `src-tauri/src/plugin.rs` — trait definition and registry

- [ ] **Step 1: Define the ToolPlugin trait**

```
// Pseudocode:

// trait ToolPlugin: Send + Sync {
//     // Return metadata about this tool
//     fn metadata() -> ToolMeta where Self: Sized;
//
//     // Detect if this tool is installed on the system
//     fn detect() -> DetectResult where Self: Sized;
//
//     // Install the tool to the given directory
//     // target_dir is the tool-specific subdirectory (e.g., <root>/claude-code/)
//     // progress is a channel sender for reporting progress
//     fn install(target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String>;
//
//     // Uninstall the tool from the given directory
//     fn uninstall(target_dir: &Path) -> Result<(), String>;
//
//     // Return the dependencies this tool requires
//     fn get_dependencies() -> Vec<ToolDependency> where Self: Sized;
// }
```

- [ ] **Step 2: Build the plugin registry**

```
// Pseudocode:

// A macro or function that registers all available plugins:
//
// fn get_all_plugins() -> Vec<Box<dyn ToolPlugin>> {
//     vec![
//         // Dependencies
//         Box::new(NodeJsPlugin),
//         Box::new(GitPlugin),
//         // CLI versions
//         Box::new(ClaudeCodeCliPlugin),
//         Box::new(CodexCliPlugin),
//         Box::new(GeminiCliPlugin),
//         Box::new(OpenCodeCliPlugin),
//         Box::new(OpenClawPlugin),
//         Box::new(HermesPlugin),
//         // Desktop versions
//         Box::new(ClaudeCodeDesktopPlugin),
//         Box::new(CodexDesktopPlugin),
//         Box::new(OpenCodeDesktopPlugin),
//     ]
// }
//
// fn get_plugin_by_id(id: &str) -> Option<Box<dyn ToolPlugin>> {
//     get_all_plugins().into_iter().find(|p| p.metadata().id == id)
// }
```

- [ ] **Step 3: Register module in lib.rs**

Add `mod plugin;` to `lib.rs`.

- [ ] **Step 4: Commit**

```
git add src-tauri/src/plugin.rs src-tauri/src/lib.rs
git commit -m "feat: add ToolPlugin trait and plugin registry"
```

---

### Task 5: Dependency Resolver

**Files:**
- Create: `src-tauri/src/services/installer/dependency_resolver.rs`

- [ ] **Step 1: Implement the dependency resolution logic**

```
// Pseudocode:

// struct DependencyResolver;

// fn resolve(tool_ids: &[String]) -> Result<InstallPlan, String>:
//   // 1. Collect all requested tools and their transitive dependencies
//   let mut all_ids = HashSet::new();
//   let mut queue = VecDeque::from(tool_ids);
//   while let Some(id) = queue.pop_front():
//     if all_ids.insert(id):
//       if let Some(plugin) = get_plugin_by_id(id):
//         for dep in plugin.get_dependencies():
//           queue.push_back(dep.tool_id)
//
//   // 2. Build dependency graph (adjacency list)
//   //    Edge: A → B means "A depends on B"
//   //    So B must be installed before A
//   let mut graph = HashMap::new();
//   for id in &all_ids:
//     if let Some(plugin) = get_plugin_by_id(id):
//       graph[id] = plugin.get_dependencies().map(|d| d.tool_id)
//
//   // 3. Topological sort (Kahn's algorithm)
//   //    Compute in-degree for each node
//   //    Queue nodes with in-degree 0
//   //    Process queue, reducing in-degree of dependents
//   //    If cycle detected → error
//   let sorted = topological_sort(&graph)?
//
//   // 4. Build install plan steps
//   //    For each tool_id in sorted order:
//   //      - detect if already installed via plugin.detect()
//   //      - if installed and version OK → step.is_installed = true
//   //      - if not installed or version too old → step.is_installed = false
//   //      - step.reason = if tool_id in original tool_ids: "selected" else: "dependency_of(<parent>)"
//   let steps = sorted.map(|id| build_step(id, &original_tool_ids))
//
//   InstallPlan { steps }
```

- [ ] **Step 2: Commit**

```
git add src-tauri/src/services/installer/dependency_resolver.rs
git commit -m "feat: add dependency resolver with topological sort"
```

---

### Task 6: Path Manager (Windows)

**Files:**
- Create: `src-tauri/src/services/installer/path_manager.rs`

- [ ] **Step 1: Implement PATH and shim management**

```
// Pseudocode:

// struct PathManager { root_dir: PathBuf }

// fn new(root_dir: &Path) -> Self:
//   root_dir is the user's unified install root (e.g., D:\AgenticTools)

// fn ensure_bin_dir() -> Result<PathBuf, String>:
//   Create <root>/bin/ if it doesn't exist
//   Return the path

// fn register_in_path() -> Result<(), String>:
//   // On Windows: add <root>\bin to HKEY_CURRENT_USER\Environment\PATH
//   // Read current PATH from registry
//   // If <root>\bin not already in PATH, append it
//   // Write back to registry
//   // Broadcast WM_SETTINGCHANGE to notify system
//   //
//   // Use winreg crate (CC Switch already depends on it for env checking)

// fn unregister_from_path() -> Result<(), String>:
//   // Remove <root>\bin from registry PATH
//   // Only remove if no tools are still installed (check installed_tools table)

// fn create_shim(tool_id: &str, executable_name: &str) -> Result<(), String>:
//   // Create a .cmd file in <root>/bin/ named <executable_name>.cmd
//   // Content: @echo off\n"<tool_install_dir>\<executable>.exe" %*
//   // This allows the tool to be called from any terminal

// fn remove_shim(executable_name: &str) -> Result<(), String>:
//   // Delete <root>/bin/<executable_name>.cmd

// fn get_tool_install_dir(tool_id: &str) -> PathBuf:
//   // Return <root>/<tool_id>/
```

- [ ] **Step 2: Commit**

```
git add src-tauri/src/services/installer/path_manager.rs
git commit -m "feat: add Windows PATH manager with shim support"
```

---

### Task 7: Installer Service (Core Engine)

**Files:**
- Create: `src-tauri/src/services/installer/mod.rs`

- [ ] **Step 1: Implement the installer service**

```
// Pseudocode:

// struct InstallerService {
//     root_path: PathBuf,
//     path_manager: PathManager,
// }

// fn new(root_path: &Path) -> Self:
//   root_path is the user's unified install root

// fn check_network() -> NetworkStatus:
//   // Try HTTPS connection to github.com
//   //   reqwest::get("https://github.com") with 5s timeout
//   // Try HTTPS connection to registry.npmjs.org
//   //   reqwest::get("https://registry.npmjs.org") with 5s timeout
//   // Return NetworkStatus with reachability for each

// fn resolve_install_plan(tool_ids: Vec<String>) -> Result<InstallPlan, String>:
//   // Delegate to DependencyResolver::resolve(&tool_ids)
//   // Return the install plan for frontend display

// fn execute_install_plan(plan: &InstallPlan) -> Result<(), String>:
//   // For each step in plan.steps:
//   //   If step.is_installed:
//   //     emit progress event: phase="skipped", message="already installed"
//   //     continue
//   //
//   //   Emit progress event: phase="starting", percent=0
//   //
//   //   Get the plugin for step.tool_id
//   //   Create target_dir = <root>/<step.tool_id>/
//   //   Create progress channel (tokio::mpsc::channel)
//   //   Spawn: plugin.install(&target_dir, progress_sender)
//   //   Listen: for each progress update, emit Tauri event "install-progress"
//   //   Wait for install to complete
//   //
//   //   If success:
//   //     run plugin.detect() to get installed version
//   //     create shim via path_manager
//   //     upsert installed_tools record with status="installed"
//   //     emit progress event: phase="complete"
//   //   If error:
//   //     upsert installed_tools record with status="error"
//   //     emit progress event: phase="error", message=error
//   //     return error

// fn uninstall_tool(tool_id: &str) -> Result<(), String>:
//   // Look up tool in installed_tools table
//   // If not found → error
//   // If category == "dependency":
//   //   Check if any other installed tools depend on this
//   //   If yes → warn user but still allow uninstall (prompt in frontend)
//   // Get plugin for tool_id
//   // Call plugin.uninstall(&install_path)
//   // Remove shim via path_manager
//   // Delete install_path directory
//   // Delete record from installed_tools table

// fn get_installed_tools() -> Result<Vec<InstalledTool>, String>:
//   // Query installed_tools table, return all rows

// fn check_tool_updates() -> Result<Vec<ToolUpdateInfo>, String>:
//   // For each installed tool with category="tool":
//   //   Get plugin, call detect() to see latest available version
//   //   Compare with installed version
//   //   If newer available → add to update list
//   // Return list

// Tauri event emission:
//   emit "install-progress" with InstallProgress payload
//   emit "install-complete" with tool_id
//   emit "install-error" with { tool_id, error }
```

- [ ] **Step 2: Register installer service module**

Create `src-tauri/src/services/installer/mod.rs` that re-exports sub-modules. Add `pub mod installer;` to `src-tauri/src/services/mod.rs`.

- [ ] **Step 3: Commit**

```
git add src-tauri/src/services/installer/ src-tauri/src/services/mod.rs
git commit -m "feat: add InstallerService with network check, install, uninstall, progress"
```

---

### Task 8: Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/tools.rs`

- [ ] **Step 1: Implement Tauri commands**

```
// Pseudocode:

// #[tauri::command]
// fn check_network() -> Result<NetworkStatus, String>:
//   InstallerService::check_network()

// #[tauri::command]
// fn resolve_install_plan(tool_ids: Vec<String>) -> Result<InstallPlan, String>:
//   DependencyResolver::resolve(&tool_ids)

// #[tauri::command]
// async fn execute_install_plan(
//     root_path: String,
//     app_handle: tauri::AppHandle,
// ) -> Result<(), String>:
//   let service = InstallerService::new(Path::new(&root_path))
//   // First ensure the bin dir and PATH are set up
//   service.path_manager.ensure_bin_dir()?
//   service.path_manager.register_in_path()?
//   // Execute the plan stored in app state (set by resolve_install_plan)
//   service.execute_install_plan(&plan)?  // progress events emitted via app_handle
//   Ok(())

// #[tauri::command]
// fn uninstall_tool(tool_id: String, root_path: String) -> Result<(), String>:
//   let service = InstallerService::new(Path::new(&root_path))
//   service.uninstall_tool(&tool_id)

// #[tauri::command]
// fn get_installed_tools(state: State<AppState>) -> Result<Vec<InstalledTool>, String>:
//   // Query from DB via AppState
//   state.db.get_installed_tools().map_err(|e| e.to_string())

// #[tauri::command]
// fn has_any_installed_tools(state: State<AppState>) -> Result<bool, String>:
//   // Used by frontend to decide: show wizard or manager?
//   state.db.has_any_installed_tools().map_err(|e| e.to_string())

// #[tauri::command]
// fn check_tool_updates(state: State<AppState>) -> Result<Vec<ToolUpdateInfo>, String>:
//   InstallerService::check_tool_updates(&state.db)
```

- [ ] **Step 2: Register commands in `commands/mod.rs`**

Add `mod tools;` and `pub use tools::*;` to `commands/mod.rs`.

- [ ] **Step 3: Register in lib.rs's tauri::Builder**

In the `run()` function, add all tool commands to the `.invoke_handler(tauri::generate_handler![...])` call.

- [ ] **Step 4: Commit**

```
git add src-tauri/src/commands/tools.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands for tool install/uninstall/detect"
```

---

### Task 9: Dependency Plugins (Node.js, Git)

**Files:**
- Create: `src-tauri/src/plugins/nodejs.rs`
- Create: `src-tauri/src/plugins/git.rs`
- Create: `src-tauri/src/plugins/mod.rs`

- [ ] **Step 1: Implement Node.js plugin**

```
// Pseudocode - NodeJsPlugin implements ToolPlugin:

// fn metadata() -> ToolMeta:
//   id: "nodejs", name: "Node.js", category: "dependency"

// fn detect() -> DetectResult:
//   // Run: node --version
//   // If success → parse version, installed=true
//   // Also check: where node → check if in our managed path or system path
//   // If not found → installed=false

// fn get_dependencies() -> Vec<ToolDependency>:
//   // Node.js has no dependencies
//   vec![]

// fn install(target_dir: &Path, progress: Sender<InstallProgress>):
//   // Download Node.js official installer:
//   //   URL: https://nodejs.org/dist/v{latest_lts}/node-v{latest_lts}-x64.msi
//   //   Download to temp file, report download progress pct
//   //   Run: msiexec /i <msi_path> /qn INSTALLDIR=<target_dir>
//   //   Wait for install to complete
//   //   Run: <target_dir>/node --version to verify
//   //   Report completion

// fn uninstall(target_dir: &Path) -> Result<(), String>:
//   // Run: msiexec /x <product_code> /qn
//   // Or simply delete the directory if it was a portable install
```

- [ ] **Step 2: Implement Git plugin**

```
// Pseudocode - GitPlugin implements ToolPlugin:

// fn metadata() -> ToolMeta:
//   id: "git", name: "Git", category: "dependency"

// fn detect() -> DetectResult:
//   // Run: git --version
//   // If success → installed=true, parse version

// fn get_dependencies() -> Vec<ToolDependency>:
//   vec![]

// fn install(target_dir: &Path, progress: Sender<InstallProgress>):
//   // Download Git for Windows:
//   //   URL: https://github.com/git-for-windows/git/releases/download/v{version}.windows.1/Git-{version}-64-bit.exe
//   //   Download to temp file
//   //   Run: <installer.exe> /VERYSILENT /NORESTART /DIR=<target_dir>
//   //   Wait and verify

// fn uninstall(target_dir: &Path) -> Result<(), String>:
//   // Find uninstaller in target_dir and run silently
//   // Or delete directory
```

- [ ] **Step 3: Create plugins module and register**

Add `src-tauri/src/plugins/mod.rs` that re-exports all plugins. Add `mod plugins;` to `lib.rs`. Update plugin registry in `plugin.rs` to include `NodeJsPlugin` and `GitPlugin`.

- [ ] **Step 4: Commit**

```
git add src-tauri/src/plugins/ src-tauri/src/plugin.rs src-tauri/src/lib.rs
git commit -m "feat: add Node.js and Git dependency plugins"
```

---

### Task 10: AI Tool Plugins — CLI (Claude Code CLI, Codex CLI, Gemini CLI)

**Files:**
- Create: `src-tauri/src/plugins/claude_code_cli.rs`
- Create: `src-tauri/src/plugins/codex_cli.rs`
- Create: `src-tauri/src/plugins/gemini_cli.rs`

- [ ] **Step 1: Implement Claude Code CLI plugin**

```
// Pseudocode - ClaudeCodeCliPlugin implements ToolPlugin:

// fn metadata() →
//   id: "claude-code-cli", name: "Claude Code (CLI)", category: "ai-cli"

// fn detect() → Run: claude --version

// fn get_dependencies() → [depends on "nodejs" >= 18]
//   CLI version installs via npm, needs Node.js

// fn install(target_dir, progress):
//   1. Set npm prefix to target_dir
//   2. Run: npm install -g @anthropic-ai/claude-code --prefix <target_dir>
//   3. Verify: <target_dir>/bin/claude --version
//   4. Create shim: <root>/bin/claude.cmd
//   Progress: "installing" → "configuring" → "complete"

// fn uninstall(target_dir):
//   Run: npm uninstall -g @anthropic-ai/claude-code --prefix <target_dir>
//   Remove shim
```

- [ ] **Step 2: Implement Codex CLI plugin**

```
// Pseudocode - CodexCliPlugin implements ToolPlugin:

// fn metadata() → id: "codex-cli", name: "Codex (CLI)", category: "ai-cli"
// fn detect() → Run: codex --version
// fn get_dependencies() → [depends on "nodejs" >= 18]
// fn install(target_dir, progress):
//   npm install -g @anthropic-ai/codex --prefix <target_dir>
//   Create shim at <root>/bin/codex.cmd
// fn uninstall(target_dir): npm uninstall or directory removal
```

- [ ] **Step 3: Implement Gemini CLI plugin**

```
// Pseudocode - GeminiCliPlugin implements ToolPlugin:

// fn metadata() → id: "gemini-cli", name: "Gemini CLI", category: "ai-cli"
// fn detect() → Run: gemini --version
// fn get_dependencies() → [depends on "nodejs" >= 18]
// fn install(target_dir, progress):
//   npm install -g @anthropic-ai/gemini-cli --prefix <target_dir>
//   Create shim
// fn uninstall(target_dir): npm uninstall or directory removal
```

- [ ] **Step 4: Register in plugin registry**

- [ ] **Step 5: Commit**

```
git add src-tauri/src/plugins/ src-tauri/src/plugin.rs
git commit -m "feat: add Claude Code CLI, Codex CLI, and Gemini CLI plugins"
```

---

### Task 10b: AI Tool Plugins — Desktop (Claude Code Desktop, Codex Desktop, OpenCode Desktop)

**Files:**
- Create: `src-tauri/src/plugins/claude_code_desktop.rs`
- Create: `src-tauri/src/plugins/codex_desktop.rs`
- Create: `src-tauri/src/plugins/opencode_desktop.rs`

- [ ] **Step 1: Implement Claude Code Desktop plugin**

```
// Pseudocode - ClaudeCodeDesktopPlugin implements ToolPlugin:

// fn metadata() →
//   id: "claude-code-desktop", name: "Claude Code (Desktop)", category: "ai-cli"

// fn detect() → check default install path: %LOCALAPPDATA%/Programs/Claude Code/

// fn get_dependencies() → vec![]
//   Desktop app bundles its own runtime

// fn install(target_dir, progress):
//   1. Download .exe installer from GitHub Releases or Anthropic official
//   2. Silent install: <installer.exe> /S /D=<target_dir>
//   3. Verify: check <target_dir>/Claude Code.exe exists
//   4. Create shim: <root>/bin/claude-desktop.cmd → launch desktop app
//   Progress: "downloading" → "installing" → "configuring" → "complete"

// fn uninstall(target_dir):
//   Run: <target_dir>/uninstall.exe /S
//   Or delete directory + remove shim
```

- [ ] **Step 2: Implement Codex Desktop plugin**

```
// Pseudocode - CodexDesktopPlugin implements ToolPlugin:

// fn metadata() →
//   id: "codex-desktop", name: "Codex (Desktop)", category: "ai-cli"
// fn detect() → check desktop install path
// fn get_dependencies() → vec![]
// fn install(target_dir, progress):
//   Download from GitHub Releases or winget
//   Silent install to target_dir, verify, create shim
// fn uninstall(target_dir): run uninstaller or delete directory
```

- [ ] **Step 3: Implement OpenCode Desktop plugin**

```
// Pseudocode - OpenCodeDesktopPlugin implements ToolPlugin:

// fn metadata() →
//   id: "opencode-desktop", name: "OpenCode (Desktop)", category: "ai-cli"
// fn detect() → check desktop install path
// fn get_dependencies() → vec![]
// fn install(target_dir, progress):
//   Download from GitHub Releases, extract/install to target_dir
//   Verify executable, create shim
// fn uninstall(target_dir): delete files and directory
```

- [ ] **Step 4: Register in plugin registry**

- [ ] **Step 5: Commit**

```
git add src-tauri/src/plugins/ src-tauri/src/plugin.rs
git commit -m "feat: add Claude Code Desktop, Codex Desktop, and OpenCode Desktop plugins"
```

---

### Task 11: AI Tool Plugins (OpenCode CLI, OpenClaw, Hermes Web UI)

**Files:**
- Create: `src-tauri/src/plugins/opencode_cli.rs`
- Create: `src-tauri/src/plugins/openclaw.rs`
- Create: `src-tauri/src/plugins/hermes.rs`

- [ ] **Step 1: Implement OpenCode CLI plugin**

```
// Pseudocode - OpenCodeCliPlugin implements ToolPlugin:

// fn metadata() → id: "opencode-cli", name: "OpenCode (CLI)", category: "ai-cli"
// fn detect() → Run: opencode --version
// fn get_dependencies() → [depends on "nodejs" >= 18]
// fn install(target_dir, progress):
//   npm install -g opencode --prefix <target_dir>
//   Create shim at <root>/bin/opencode.cmd
// fn uninstall(target_dir): npm uninstall or directory cleanup
```

- [ ] **Step 2: Implement OpenClaw plugin**

```
// Pseudocode - OpenClawPlugin implements ToolPlugin:
//   id: "openclaw", name: "OpenClaw", category: "ai-cli"
//   detect() → Run: openclaw --version
//   get_dependencies() → [depends on "nodejs" >= 18]
//   install() → Download binary from GitHub Release, extract, create shim
//   uninstall() → Remove binary and directory
```

- [ ] **Step 3: Implement Hermes Web UI plugin**

```
// Pseudocode - HermesPlugin implements ToolPlugin:

// fn metadata() → id: "hermes", name: "Hermes (Web UI)", category: "ai-cli"

// fn detect() → Run: hermes --version
//   Also check if Web UI is reachable at default port

// fn get_dependencies() → [depends on "nodejs" >= 18]

// fn install(target_dir, progress):
//   1. Set npm prefix to target_dir
//   2. Run: npm install -g hermes --prefix <target_dir>
//   3. Verify: <target_dir>/bin/hermes --version
//   4. Create two shims:
//      <root>/bin/hermes.cmd → standard CLI entry
//      <root>/bin/hermes-webui.cmd → launches Web UI
//   Progress: "installing" → "configuring" → "complete"

// fn uninstall(target_dir):
//   Run: npm uninstall -g hermes --prefix <target_dir>
//   Remove both hermes.cmd and hermes-webui.cmd shims
```

- [ ] **Step 4: Register in plugin registry**

- [ ] **Step 5: Commit**

```
git add src-tauri/src/plugins/ src-tauri/src/plugin.rs
git commit -m "feat: add OpenCode CLI, OpenClaw, and Hermes Web UI plugins"
```

---

### Task 12: Store and State Wiring

**Files:**
- Modify: `src-tauri/src/store.rs` — add installer-related state
- Modify: `src-tauri/src/lib.rs` — initialize services in setup

- [ ] **Step 1: Store the install root path as a setting**

Reuse CC Switch's existing `settings` key-value table. On first launch, save the user's chosen install root path:

```
// Pseudocode:
// In Database impl:
//   fn get_install_root() -> Option<String>:
//     SELECT value FROM settings WHERE key = 'install_root'
//   fn set_install_root(path: &str):
//     INSERT OR REPLACE INTO settings (key, value) VALUES ('install_root', ?)
```

No changes to `AppState` struct needed — the install root is read from DB settings on each operation, not held in memory.

- [ ] **Step 2: Register all commands in lib.rs setup**

In the `run()` function's `tauri::Builder`, add the new commands to `invoke_handler` and ensure plugin initialization runs:

```
// Pseudocode - in run() setup closure:
//   // No persistent state needed beyond DB
//   // InstallerService is created per-operation with the current root_path
```

- [ ] **Step 3: Commit**

```
git add src-tauri/src/store.rs src-tauri/src/lib.rs
git commit -m "feat: wire installer state and commands into app setup"
```

---

### Task 13: Frontend Type Definitions

**Files:**
- Create: `src/types/tools.ts`

- [ ] **Step 1: Define TypeScript types matching Rust types**

```
// Pseudocode (TypeScript types):

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

- [ ] **Step 2: Commit**

```
git add src/types/tools.ts
git commit -m "feat: add frontend TypeScript types for tool management"
```

---

### Task 14: Frontend API Layer

**Files:**
- Create: `src/lib/api/tools.ts`

- [ ] **Step 1: Create tools API module**

```
// Pseudocode (API functions):

// import { invoke } from '@tauri-apps/api/core'
// import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// export const toolsApi = {
//   checkNetwork(): Promise<NetworkStatus>:
//     invoke('check_network')

//   resolveInstallPlan(toolIds: string[]): Promise<InstallPlan>:
//     invoke('resolve_install_plan', { toolIds })

//   executeInstallPlan(rootPath: string): Promise<void>:
//     invoke('execute_install_plan', { rootPath })

//   uninstallTool(toolId: string, rootPath: string): Promise<void>:
//     invoke('uninstall_tool', { toolId, rootPath })

//   getInstalledTools(): Promise<InstalledTool[]>:
//     invoke('get_installed_tools')

//   hasAnyInstalledTools(): Promise<boolean>:
//     invoke('has_any_installed_tools')

//   checkToolUpdates(): Promise<ToolUpdateInfo[]>:
//     invoke('check_tool_updates')

//   // Event listeners
//   onInstallProgress(callback: (progress: InstallProgress) => void): Promise<UnlistenFn>:
//     listen<InstallProgress>('install-progress', (event) => callback(event.payload))

//   onInstallComplete(callback: (toolId: string) => void): Promise<UnlistenFn>:
//     listen<string>('install-complete', (event) => callback(event.payload))

//   onInstallError(callback: (error: { toolId: string, error: string }) => void): Promise<UnlistenFn>:
//     listen('install-error', (event) => callback(event.payload as any))
// }
```

- [ ] **Step 2: Commit**

```
git add src/lib/api/tools.ts
git commit -m "feat: add frontend API bindings for tool commands and events"
```

---

### Task 15: React Query Hooks for Tools

**Files:**
- Create: `src/hooks/useTools.ts`
- Create: `src/hooks/useInstallProgress.ts`

- [ ] **Step 1: Create useTools hook**

```
// Pseudocode:

// use @tanstack/react-query (useQuery, useMutation, useQueryClient)

// export function useInstalledTools():
//   useQuery({
//     queryKey: ['installed-tools'],
//     queryFn: () => toolsApi.getInstalledTools(),
//     staleTime: 30_000  // 30s cache
//   })

// export function useHasInstalledTools():
//   useQuery({
//     queryKey: ['has-installed-tools'],
//     queryFn: () => toolsApi.hasAnyInstalledTools()
//   })

// export function useCheckNetwork():
//   useQuery({
//     queryKey: ['network-status'],
//     queryFn: () => toolsApi.checkNetwork(),
//     retry: false,
//     refetchOnWindowFocus: true
//   })

// export function useResolveInstallPlan():
//   useMutation({
//     mutationFn: (toolIds: string[]) => toolsApi.resolveInstallPlan(toolIds)
//   })

// export function useExecuteInstallPlan():
//   useMutation({
//     mutationFn: (rootPath: string) => toolsApi.executeInstallPlan(rootPath),
//     onSuccess: () => queryClient.invalidateQueries({ queryKey: ['installed-tools'] })
//   })

// export function useUninstallTool():
//   useMutation({
//     mutationFn: ({ toolId, rootPath }: { toolId: string, rootPath: string }) =>
//       toolsApi.uninstallTool(toolId, rootPath),
//     onSuccess: () => queryClient.invalidateQueries({ queryKey: ['installed-tools'] })
//   })
```

- [ ] **Step 2: Create useInstallProgress hook**

```
// Pseudocode:

// export function useInstallProgress():
//   const [progressMap, setProgressMap] = useState<Map<string, InstallProgress>>(new Map())
//
//   useEffect:
//     const unlisten = toolsApi.onInstallProgress((progress) => {
//       setProgressMap(prev => {
//         const next = new Map(prev)
//         next.set(progress.toolId, progress)
//         return next
//       })
//     })
//     return () => { unlisten.then(fn => fn()) }
//
//   const getToolProgress = (toolId: string) => progressMap.get(toolId) ?? null
//   const allComplete = [...progressMap.values()].every(p =>
//     p.phase === 'complete' || p.phase === 'skipped' || p.phase === 'error'
//   )
//   const hasErrors = [...progressMap.values()].some(p => p.phase === 'error')
//
//   return { progressMap, getToolProgress, allComplete, hasErrors }
```

- [ ] **Step 3: Commit**

```
git add src/hooks/useTools.ts src/hooks/useInstallProgress.ts
git commit -m "feat: add React Query hooks for tool management and install progress"
```

---

### Task 16: EnvCheckPanel Component

**Files:**
- Create: `src/components/tools/EnvCheckPanel.tsx`

- [ ] **Step 1: Build the network check panel**

```
// Pseudocode (React component):

// Props: { onNext: () => void }

// Uses useCheckNetwork() hook

// States:
//   - loading: spinner + "检测网络连接..."
//   - error: warning icon + "网络连接异常"
//     Show: "请先解决网络问题再继续安装。"
//     Link to external guide (GitHub Wiki URL for network troubleshooting)
//     Retry button
//   - success: check icon + "网络连接正常"
//     Auto-transition to next step after 1s, or show "下一步" button

// Visual layout:
//   Centered card with:
//     icon (Loader2 / CheckCircle / AlertTriangle)
//     status text
//     action button (retry or next)
```

- [ ] **Step 2: Commit**

```
git add src/components/tools/EnvCheckPanel.tsx
git commit -m "feat: add network status check panel component"
```

---

### Task 17: PathConfig Component

**Files:**
- Create: `src/components/tools/PathConfig.tsx`

- [ ] **Step 1: Build the path configuration component**

```
// Pseudocode (React component):

// Props: { onNext: (rootPath: string) => void, onBack: () => void }

// State:
//   rootPath: string (default: "C:\\AgenticTools" on Windows)

// UI:
//   Input field with label "安装根目录"
//   Browse button → use Tauri dialog plugin to pick directory
//   Preview section showing what the directory tree will look like:
//     C:\AgenticTools\
//       bin\           ← shim scripts
//       claude-code\
//       codex\
//       gemini-cli\
//       ...
//   "上一步" and "下一步" buttons
//   Validation: path must exist or be creatable, not a system directory

// On "下一步": call onNext(rootPath)
```

- [ ] **Step 2: Commit**

```
git add src/components/tools/PathConfig.tsx
git commit -m "feat: add install root path configuration component"
```

---

### Task 18: InstallProgress Component

**Files:**
- Create: `src/components/tools/InstallProgress.tsx`

- [ ] **Step 1: Build the installation progress display**

```
// Pseudocode (React component):

// Props: {
//   installPlan: InstallPlan,
//   onComplete: () => void,
//   onError: (toolId: string, error: string) => void
// }

// Uses useInstallProgress() hook

// UI:
//   List of steps from installPlan.steps[], each row shows:
//     - Tool icon/name
//     - Category badge ("已选择" | "依赖" | "已安装")
//     - Status indicator per row:
//         pending:   gray dot
//         active:    spinning loader + progress bar (0-100%)
//         complete:  green checkmark
//         skipped:   gray "已安装" tag
//         error:     red X + error message
//     - Phase text: "下载中..." / "安装中..." / "配置中..." / "完成"

//   Overall progress bar at top:
//     (completed steps + skipped) / total steps * 100%

//   When allComplete:
//     Show "安装完成！" with confetti or success animation
//     "进入管理" button → calls onComplete()

//   When hasErrors:
//     Show which tools failed + error messages
//     "重试失败项" button
```

- [ ] **Step 2: Commit**

```
git add src/components/tools/InstallProgress.tsx
git commit -m "feat: add installation progress display component"
```

---

### Task 19: Wizard Page

**Files:**
- Create: `src/pages/Wizard.tsx`

- [ ] **Step 1: Build the first-run wizard page**

```
// Pseudocode (React component):

// Props: { onComplete: () => void }

// Multi-step wizard with step indicator at top:
//   Step 1: 网络检测 (EnvCheckPanel)
//   Step 2: 安装路径 (PathConfig)
//   Step 3: 选择工具 (ToolChecklist - inline, not a separate component yet)
//   Step 4: 安装中 (InstallProgress)

// State:
//   currentStep: 1 | 2 | 3 | 4
//   rootPath: string = ""
//   selectedTools: string[] = []
//   installPlan: InstallPlan | null = null

// Step 3 UI (tool selection):
//   Grid of tool cards (6 tools), each showing:
//     - Icon
//     - Name + short description
//     - Checkbox (checked by default for all)
//     - "依赖: Node.js" label
//   "全部勾选" / "全部取消" toggle
//   "上一步" and "开始安装" buttons

//   On "开始安装":
//     Call resolveInstallPlan(selectedTools)
//     Show the plan (which dependencies will also be installed)
//     After user confirmation → move to step 4
//     Call executeInstallPlan(rootPath)

// Step 4: <InstallProgress plan={installPlan} onComplete={onComplete} />

// Page layout: centered card with max-w-2xl, step dots at top
```

- [ ] **Step 2: Commit**

```
git add src/pages/Wizard.tsx
git commit -m "feat: add first-run wizard page with tool selection and install flow"
```

---

### Task 20: Manager Page

**Files:**
- Create: `src/pages/Manager.tsx`

- [ ] **Step 1: Build the daily manager page**

```
// Pseudocode (React component):

// Uses: useInstalledTools(), useUninstallTool(), useCheckToolUpdates()

// UI layout:
//   Header: "工具管理" + "检查更新" button
//
//   Two-tab layout (like a software管家):
//     Tab "已安装" (default):
//       List of installed tools as cards/grid:
//         - Tool icon + name + version
//         - Install path (truncated)
//         - Status badge: "已安装" (green)
//         - Actions: "卸载" button, "更新" button (if update available)
//
//     Tab "未安装":
//       Grid of available tools that are not yet installed:
//         - Tool icon + name + description
//         - "安装" button
//         Clicking "安装" → resolve plan for that single tool →
//           show confirm dialog → install → refresh list
//
//   Each tool card uses ToolCard component (extracted next)
//
//   Uninstall flow:
//     Click "卸载" → confirm dialog
//       If category=="dependency": warn "其他工具可能依赖此项"
//       "确认卸载" → call uninstallTool(toolId, rootPath) → refresh list
//
//   Settings area (collapsible):
//     Current install root path (read-only with "修改" button → opens PathConfig)
//     "查看已安装工具列表" link
```

- [ ] **Step 2: Commit**

```
git add src/pages/Manager.tsx
git commit -m "feat: add manager page with installed/uninstalled tabs and uninstall flow"
```

---

### Task 21: ToolCard Component

**Files:**
- Create: `src/components/tools/ToolCard.tsx`

- [ ] **Step 1: Build the reusable tool card**

```
// Pseudocode (React component):

// Props: {
//   tool: InstalledTool | ToolMeta
//   variant: 'installed' | 'available'
//   onInstall?: () => void
//   onUninstall?: () => void
//   onUpdate?: () => void
//   progress?: InstallProgress  // for in-progress installations
// }

// UI:
//   Card with:
//     - Left: tool icon (use Lucide terminal/code icon or tool-specific SVG)
//     - Center:
//         tool name (bold)
//         version or description (muted, small)
//         if progress and installing: mini progress bar
//     - Right:
//         if variant=='installed':
//           "已安装 v1.2.3" badge (green)
//           "卸载" button (destructive variant)
//           "更新" button (if update available, blue outline)
//         if variant=='available':
//           "安装" button (primary)
//         if progress:
//           replace button area with progress indicator

// Uses shadcn/ui Card, Badge, Button, Progress components
```

- [ ] **Step 2: Commit**

```
git add src/components/tools/ToolCard.tsx
git commit -m "feat: add reusable ToolCard component"
```

---

### Task 22: App.tsx Integration

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Add wizard and manager views to the View type**

```
// Pseudocode - add to View type union:
//   | "wizard"
//   | "manager"

// Add to VALID_VIEWS array:
//   ..., "wizard", "manager"
```

- [ ] **Step 2: Add navigation entries**

```
// In the sidebar/toolbar navigation:
//   - Add a "装机" (or "Tools") nav item
//   - Icon: Wrench or Package icon from lucide-react
//   - On click: check hasAnyInstalledTools()
//       if false → setCurrentView("wizard")
//       if true → setCurrentView("manager")
```

- [ ] **Step 3: Add view rendering logic**

```
// In the main render switch/case:
//   case "wizard":
//     return <Wizard onComplete={() => setCurrentView("manager")} />
//   case "manager":
//     return <Manager onInstallMore={() => setCurrentView("wizard")} />
```

- [ ] **Step 4: Handle first-run logic**

```
// On app mount:
//   Check hasAnyInstalledTools()
//   If false AND no localStorage "wizard-seen" flag:
//     auto-navigate to wizard view (setCurrentView("wizard"))
//   Set localStorage flag after wizard completes
```

- [ ] **Step 5: Commit**

```
git add src/App.tsx
git commit -m "feat: integrate wizard and manager views into app navigation"
```

---

### Task 23: i18n Strings

**Files:**
- Modify: `src/i18n/` — add translation keys for the tool management UI

- [ ] **Step 1: Add Chinese (zh) translations**

```
// Add to zh translation JSON:

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

- [ ] **Step 2: Add English (en) equivalents**

Same keys with English values.

- [ ] **Step 3: Commit**

```
git add src/i18n/
git commit -m "feat: add i18n strings for tool management UI"
```

---

### Task 24: Integration Testing and Smoke Test

**Files:**
- Create: `src-tauri/tests/tool_plugins_tests.rs`

- [ ] **Step 1: Write integration tests for dependency resolution**

```
// Pseudocode - Rust integration tests:

// #[test]
// fn test_dependency_resolver_no_deps():
//   // Select a tool with no dependencies (e.g., Node.js)
//   // Plan should have exactly 1 step for Node.js

// #[test]
// fn test_dependency_resolver_with_deps():
//   // Select "Claude Code"
//   // Plan should have ["nodejs", "claude-code"] in that order
//   // Node.js reason = "dependency_of(Claude Code)"
//   // Claude Code reason = "selected"

// #[test]
// fn test_dependency_resolver_dedup():
//   // Select "Claude Code" and "Codex"
//   // Both depend on Node.js
//   // Plan should include Node.js only ONCE
//   // Order: nodejs, then claude-code, then codex (or codex then claude-code)

// #[test]
// fn test_dependency_resolver_cycle_detection():
//   // If plugins somehow have circular deps (shouldn't happen, but test it)
//   // Should return an error, not hang
```

- [ ] **Step 2: Smoke test the full flow manually**

```
// Manual verification checklist:
// 1. App starts, no installed tools → wizard shown
// 2. Network check works (pass/fail states)
// 3. Path selection works (browse dialog, validation)
// 4. Tool selection: check/uncheck tools
// 5. Install plan shown with correct dependency order
// 6. Installation runs, progress events received and displayed
// 7. After install, manager shows installed tools
// 8. Uninstall flow works (confirm → uninstall → list update)
// 9. After all uninstalled → wizard shown again
// 10. Existing CC Switch features still work (providers, proxy, settings)
```

- [ ] **Step 3: Fix issues found during smoke test**

Fix bugs incrementally, commit each fix separately.

- [ ] **Step 4: Final commit**

```
git add src-tauri/tests/ src-tauri/src/
git commit -m "test: add integration tests for dependency resolution"
```

---

### Task 25: CI / Build Verification

**Files:**
- Modify: `.github/workflows/` — check if any CI config needs updating

- [ ] **Step 1: Verify Tauri build on Windows**

```
Run: pnpm tauri build (or cargo build in src-tauri)
Expected: Successful compilation, .msi installer produced
```

- [ ] **Step 2: Update GitHub workflow if needed**

Check if CC Switch's existing CI workflows work with the new name/config. Update only if they fail — the fork should preserve the working CI setup.

- [ ] **Step 3: Commit any CI fixes**

---

## Implementation Order Notes

- Tasks 1-12 (Rust backend) can be done largely independently from tasks 13-22 (frontend)
- Tasks 5-7 are sequential: types → resolver → path manager → installer → commands
- Tasks 8-11 depend on task 4 (trait definition)
- Tasks 15 depends on task 14 (API), task 14 depends on task 13 (types)
- Tasks 16-18 (components) can be built in parallel
- Tasks 19-20 (pages) depend on components
- Task 22 (App.tsx integration) is the final frontend step
- Task 24 (testing) should run after backend + frontend are both done

## File Creation/Modification Summary

**Create (20 new files):**
| File | Purpose |
|------|---------|
| `src-tauri/src/tool_types.rs` | Shared data types |
| `src-tauri/src/plugin.rs` | ToolPlugin trait + registry |
| `src-tauri/src/services/installer/mod.rs` | Installer service facade |
| `src-tauri/src/services/installer/dependency_resolver.rs` | Topological sort |
| `src-tauri/src/services/installer/path_manager.rs` | PATH + shim management |
| `src-tauri/src/commands/tools.rs` | Tauri command handlers |
| `src-tauri/src/plugins/mod.rs` | Plugin module |
| `src-tauri/src/plugins/nodejs.rs` | Node.js plugin |
| `src-tauri/src/plugins/git.rs` | Git plugin |
| `src-tauri/src/plugins/claude_code.rs` | Claude Code plugin |
| `src-tauri/src/plugins/codex.rs` | Codex plugin |
| `src-tauri/src/plugins/gemini_cli.rs` | Gemini CLI plugin |
| `src-tauri/src/plugins/opencode.rs` | OpenCode plugin |
| `src-tauri/src/plugins/openclaw.rs` | OpenClaw plugin |
| `src-tauri/src/plugins/hermes.rs` | Hermes plugin |
| `src-tauri/src/database/dao/tools.rs` | DB queries for installed_tools |
| `src/types/tools.ts` | Frontend TS types |
| `src/lib/api/tools.ts` | Frontend API bindings |
| `src/hooks/useTools.ts` | React Query hooks |
| `src/hooks/useInstallProgress.ts` | Progress event hook |
| `src/components/tools/EnvCheckPanel.tsx` | Network check panel |
| `src/components/tools/PathConfig.tsx` | Path config component |
| `src/components/tools/InstallProgress.tsx` | Progress display |
| `src/components/tools/ToolCard.tsx` | Reusable tool card |
| `src/pages/Wizard.tsx` | First-run wizard |
| `src/pages/Manager.tsx` | Daily manager |
| `src-tauri/tests/tool_plugins_tests.rs` | Integration tests |

**Modify (7 existing files):**
| File | Change |
|------|--------|
| `package.json` | Metadata |
| `src-tauri/Cargo.toml` | Metadata |
| `src-tauri/tauri.conf.json` | Product name/identifier |
| `README.md` | AgenticBoot content |
| `src-tauri/src/lib.rs` | Add mod declarations, register commands |
| `src-tauri/src/database/schema.rs` | v11 migration |
| `src-tauri/src/database/mod.rs` | SCHEMA_VERSION bump |
| `src-tauri/src/database/dao/mod.rs` | Register tools DAO |
| `src-tauri/src/services/mod.rs` | Register installer module |
| `src-tauri/src/commands/mod.rs` | Register tools commands |
| `src/App.tsx` | Add wizard/manager views |
| `src/i18n/` | Add translation keys |
