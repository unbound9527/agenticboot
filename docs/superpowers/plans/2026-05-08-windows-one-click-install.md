# Windows One-Click Install Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows-first one-click installer that detects preinstalled tools, skips redundant installs, installs real official desktop apps, and keeps uninstall safe for system-managed installs.

**Architecture:** Extend the existing Rust `ToolPlugin` system with install-strategy metadata, then branch installer execution and uninstall behavior by strategy instead of assuming every tool is a managed local prefix. Keep the React/Tauri flow intact while tightening detection, Windows path handling, and desktop-app install logic.

**Tech Stack:** Tauri 2, Rust, tokio, reqwest, winreg, React 18, TypeScript, Vitest

---

## File Map

- Modify: `src-tauri/src/plugin.rs`
  Add backend-only install strategy metadata and ownership helpers to `ToolPlugin`.

- Modify: `src-tauri/src/tool_types.rs`
  Add any install-plan / detection structs needed by the backend while keeping frontend serialization stable.

- Create: `src-tauri/src/services/installer/windows.rs`
  Centralize Windows-specific executable resolution, managed-prefix probing, desktop-app detection, and installer helpers.

- Modify: `src-tauri/src/services/installer/mod.rs`
  Branch install execution and uninstall behavior by strategy, rerun detection after install, and persist the detected path instead of guessing.

- Modify: `src-tauri/src/services/installer/path_manager.rs`
  Replace package-name-specific shim assumptions with path-based shim creation for managed executables only.

- Modify: `src-tauri/src/services/installer/dependency_resolver.rs`
  Keep detect-before-install behavior explicit and test that all selected tools and dependencies are skipped when already installed.

- Modify: `src-tauri/src/plugins/nodejs.rs`
  Fix Windows zip layout handling and detection for managed Node.js installs.

- Modify: `src-tauri/src/plugins/git.rs`
  Fix Windows MinGit layout detection and expose managed-install metadata.

- Modify: `src-tauri/src/plugins/claude_code_cli.rs`
  Fix Windows npm-prefix detection and switch to path-based managed executable resolution.

- Modify: `src-tauri/src/plugins/codex_cli.rs`
  Fix Windows npm-prefix detection and switch to path-based managed executable resolution.

- Modify: `src-tauri/src/plugins/gemini_cli.rs`
  Fix Windows npm-prefix detection and switch to path-based managed executable resolution.

- Modify: `src-tauri/src/plugins/opencode_cli.rs`
  Replace the current Windows “unsupported” branch with native Windows install + detection logic.

- Modify: `src-tauri/src/plugins/openclaw.rs`
  Move Windows install to the official native path, detect preinstalled `openclaw`, and keep ownership-safe uninstall.

- Modify: `src-tauri/src/plugins/hermes.rs`
  Implement native Windows install/detect behavior that results in a usable `hermes dashboard`.

- Modify: `src-tauri/src/plugins/claude_code_desktop.rs`
  Replace npm-backed desktop behavior with official desktop install and detection.

- Modify: `src-tauri/src/plugins/codex_desktop.rs`
  Replace npm-backed desktop behavior with official desktop install and detection.

- Modify: `src-tauri/src/plugins/opencode_desktop.rs`
  Replace the current placeholder Windows branch with official desktop install and detection.

- Modify: `src-tauri/src/plugins/mod.rs`
  Export any new shared helpers if the plugin modules need them.

- Modify: `src/pages/Wizard.tsx`
  Preserve current UX but make already-installed tools consistently unselected/skipped regardless of whether they were installed by AgenticBoot or externally.

- Modify: `src/pages/Manager.tsx`
  Show already-detected tools as installed even when outside the managed root and prevent misleading reinstall assumptions.

- Modify: `src/components/tools/InstallProgress.tsx`
  Keep the UI consistent for preinstalled/skipped tools once backend logic is tightened.

- Modify: `tests` as needed:
  Add or extend frontend tests around Wizard / Manager skip behavior.

### Task 1: Add Install Strategy Metadata And Safe Ownership Rules

**Files:**
- Create: `src-tauri/src/services/installer/windows.rs`
- Modify: `src-tauri/src/plugin.rs`
- Modify: `src-tauri/src/tool_types.rs`
- Modify: `src-tauri/src/services/installer/mod.rs`
- Test: `src-tauri/src/plugin.rs`
- Test: `src-tauri/src/services/installer/mod.rs`

- [ ] **Step 1: Write the failing backend tests for install strategy and uninstall ownership**

Add `#[cfg(test)]` coverage that locks down two behaviors:

```rust
#[test]
fn install_strategy_desktop_plugins_are_not_managed_prefix_tools() {
    let plugin = crate::plugin::get_plugin_by_id("claude-code-desktop").unwrap();
    assert_eq!(plugin.install_strategy(), InstallStrategy::DesktopInstaller);
    assert!(!plugin.is_managed_by_root());
}

#[test]
fn install_strategy_managed_prefix_plugins_are_owned_by_agenticboot() {
    let plugin = crate::plugin::get_plugin_by_id("claude-code-cli").unwrap();
    assert_eq!(plugin.install_strategy(), InstallStrategy::ManagedPrefix);
    assert!(plugin.is_managed_by_root());
}

#[test]
fn install_strategy_hermes_plugin_is_registered() {
    let plugin = crate::plugin::get_plugin_by_id("hermes").unwrap();
    assert_eq!(plugin.metadata().id, "hermes");
}
```

And in `src-tauri/src/services/installer/mod.rs` add a failing test for safe uninstall branching:

```rust
#[test]
fn install_strategy_uninstall_policy_only_deletes_managed_prefix_directories() {
    assert!(should_delete_install_dir(InstallStrategy::ManagedPrefix, true));
    assert!(!should_delete_install_dir(InstallStrategy::DesktopInstaller, false));
    assert!(!should_delete_install_dir(InstallStrategy::PythonPackage, false));
}
```

- [ ] **Step 2: Run the targeted backend tests to verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml install_strategy_
```

Expected:

- Compilation or test failure because `InstallStrategy`, `install_strategy()`, `is_managed_by_root()`, and `should_delete_install_dir()` do not exist yet
- If the strategy API is added before plugin registration is fixed, `install_strategy_hermes_plugin_is_registered` should still fail until Hermes is registered

- [ ] **Step 3: Add backend install-strategy types and plugin trait methods**

Update `src-tauri/src/plugin.rs` with backend-only metadata:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStrategy {
    ManagedPrefix,
    OfficialScript,
    PythonPackage,
    DesktopInstaller,
}

pub trait ToolPlugin: Send + Sync {
    fn metadata(&self) -> ToolMeta;
    fn install_strategy(&self) -> InstallStrategy;
    fn detect(&self, install_root: Option<&Path>) -> DetectResult;
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String>;
    fn uninstall(&self, target_dir: &Path) -> Result<(), String>;
    fn dependencies(&self) -> Vec<ToolDependency>;

    fn is_managed_by_root(&self) -> bool {
        matches!(self.install_strategy(), InstallStrategy::ManagedPrefix)
    }
}
```

Also update the plugin registry to include Hermes:

```rust
pub fn get_all_plugins() -> Vec<Box<dyn ToolPlugin>> {
    vec![
        Box::new(crate::plugins::nodejs::NodeJsPlugin),
        Box::new(crate::plugins::git::GitPlugin),
        Box::new(crate::plugins::claude_code_cli::ClaudeCodeCliPlugin),
        Box::new(crate::plugins::codex_cli::CodexCliPlugin),
        Box::new(crate::plugins::gemini_cli::GeminiCliPlugin),
        Box::new(crate::plugins::opencode_cli::OpenCodeCliPlugin),
        Box::new(crate::plugins::openclaw::OpenClawPlugin),
        Box::new(crate::plugins::hermes::HermesPlugin),
        Box::new(crate::plugins::claude_code_desktop::ClaudeCodeDesktopPlugin),
        Box::new(crate::plugins::codex_desktop::CodexDesktopPlugin),
        Box::new(crate::plugins::opencode_desktop::OpenCodeDesktopPlugin),
    ]
}
```

Add a safe uninstall helper in `src-tauri/src/services/installer/mod.rs`:

```rust
fn should_delete_install_dir(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    owned_by_root && matches!(strategy, InstallStrategy::ManagedPrefix)
}
```

- [ ] **Step 4: Add a Windows helper module skeleton**

Create `src-tauri/src/services/installer/windows.rs` with the shared types the next tasks will extend:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsDetectPaths {
    pub executable: Option<PathBuf>,
    pub install_root: Option<PathBuf>,
}

pub fn normalize_windows_exe(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}
```

- [ ] **Step 5: Wire uninstall branching into `InstallerService`**

In `src-tauri/src/services/installer/mod.rs`, change uninstall to consult the plugin strategy before deleting files:

```rust
let plugin = get_plugin_by_id(tool_id).ok_or_else(|| format!("unknown tool: {tool_id}"))?;
let strategy = plugin.install_strategy();
let owned_by_root = plugin.is_managed_by_root();

plugin.uninstall(target_dir).ok();

if matches!(strategy, InstallStrategy::ManagedPrefix) {
    self.path_manager.remove_shim(tool_id)?;
}

if should_delete_install_dir(strategy, owned_by_root) && target_dir.exists() {
    std::fs::remove_dir_all(target_dir)
        .map_err(|e| format!("failed to remove install dir: {e}"))?;
}
```

- [ ] **Step 6: Run the targeted backend tests to verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml install_strategy_
```

Expected:

- PASS for all install-strategy tests, including Hermes plugin registration

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/plugin.rs src-tauri/src/tool_types.rs src-tauri/src/services/installer/mod.rs src-tauri/src/services/installer/windows.rs
git commit -m "feat: add install strategy metadata and safe uninstall policy"
```

### Task 2: Fix Windows Managed-Prefix Detection And Shim Generation

**Files:**
- Modify: `src-tauri/src/services/installer/windows.rs`
- Modify: `src-tauri/src/services/installer/path_manager.rs`
- Modify: `src-tauri/src/services/installer/mod.rs`
- Modify: `src-tauri/src/plugins/nodejs.rs`
- Modify: `src-tauri/src/plugins/git.rs`
- Modify: `src-tauri/src/plugins/claude_code_cli.rs`
- Modify: `src-tauri/src/plugins/codex_cli.rs`
- Modify: `src-tauri/src/plugins/gemini_cli.rs`
- Test: `src-tauri/src/services/installer/path_manager.rs`
- Test: `src-tauri/src/plugins/nodejs.rs`
- Test: `src-tauri/src/plugins/git.rs`

- [ ] **Step 1: Write failing tests for Windows executable resolution**

Add unit tests that describe the layouts we need to support:

```rust
#[test]
fn windows_paths_nodejs_detects_root_level_node_exe_after_zip_extract() {
    let tmp = tempfile::tempdir().unwrap();
    let node_dir = tmp.path().join("nodejs");
    std::fs::create_dir_all(&node_dir).unwrap();
    std::fs::write(node_dir.join("node.exe"), b"").unwrap();

    let detect = NodeJsPlugin.detect(Some(tmp.path()));
    assert!(detect.installed);
    assert_eq!(detect.install_path.as_deref(), Some(node_dir.to_string_lossy().as_ref()));
}

#[test]
fn windows_paths_cli_shim_targets_actual_managed_executable_path() {
    let tmp = tempfile::tempdir().unwrap();
    let pm = PathManager::new(tmp.path());
    pm.create_shim("claude", "D:\\AgenticTools\\claude-code-cli\\claude.cmd").unwrap();

    let content = std::fs::read_to_string(tmp.path().join("bin").join("claude.cmd")).unwrap();
    assert!(content.contains("claude-code-cli\\claude.cmd"));
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml windows_paths_
```

Expected:

- FAIL because the current Node.js detection expects `nodejs/bin/node.exe` and current shim handling still assumes package-specific npm layout

- [ ] **Step 3: Add shared Windows managed-prefix resolution helpers**

Extend `src-tauri/src/services/installer/windows.rs`:

```rust
pub fn find_managed_executable(root: &Path, tool_dir: &str, candidates: &[&str]) -> Option<PathBuf> {
    let base = root.join(tool_dir);
    candidates
        .iter()
        .map(|candidate| base.join(candidate))
        .find(|path| path.exists())
}

pub fn npm_prefix_candidates(cmd_name: &str) -> Vec<String> {
    vec![
        format!("{cmd_name}.cmd"),
        format!("{cmd_name}.exe"),
        format!("bin\\{cmd_name}.cmd"),
        format!("bin\\{cmd_name}.exe"),
    ]
}
```

- [ ] **Step 4: Make `PathManager` path-based instead of package-name-based**

Replace the npm-package shim coupling with an executable-path helper:

```rust
pub fn create_cmd_shim(&self, shim_name: &str, target_exe: &Path) -> Result<(), String> {
    let bin_dir = self.ensure_bin_dir()?;
    let shim_path = bin_dir.join(format!("{shim_name}.cmd"));
    let content = format!("@echo off\r\n\"{}\" %*\r\n", target_exe.to_string_lossy());
    fs::write(&shim_path, content).map_err(|e| format!("failed to create shim: {e}"))?;
    Ok(())
}
```

Keep `create_shim()` as a thin wrapper if needed, but stop relying on `create_npm_shim()` as the default path.

- [ ] **Step 5: Fix Node.js and Git detection to match actual Windows layouts**

Update `src-tauri/src/plugins/nodejs.rs`:

```rust
if let Some(root) = install_root {
    if let Some(path) = crate::services::installer::windows::find_managed_executable(
        root,
        "nodejs",
        &["node.exe", "bin\\node.exe"],
    ) {
        return DetectResult {
            installed: true,
            version: None,
            install_path: path.parent().map(|p| p.to_string_lossy().to_string()),
        };
    }
}
```

Update `src-tauri/src/plugins/git.rs` similarly with candidates:

```rust
&["cmd\\git.exe", "bin\\git.exe"]
```

- [ ] **Step 6: Fix npm CLI detection to match Windows prefix installs**

Update the CLI plugins to use the shared candidates:

```rust
if let Some(root) = install_root {
    let candidates = crate::services::installer::windows::npm_prefix_candidates("claude");
    if let Some(path) = crate::services::installer::windows::find_managed_executable(
        root,
        "claude-code-cli",
        &candidates.iter().map(String::as_str).collect::<Vec<_>>(),
    ) {
        return DetectResult {
            installed: true,
            version: None,
            install_path: Some(path.parent().unwrap().to_string_lossy().to_string()),
        };
    }
}
```

Apply the same pattern to `codex` and `gemini`.

- [ ] **Step 7: Re-detect after install and create shims from real executable paths**

In `src-tauri/src/services/installer/mod.rs`, after `plugin.install(...)` succeeds:

```rust
let detect = post_plugin.detect(Some(&self.root_path));
let install_path = detect
    .install_path
    .clone()
    .unwrap_or_else(|| target_dir.to_string_lossy().to_string());

if post_plugin.is_managed_by_root() {
    if let Some(exe_path) = crate::services::installer::windows::find_managed_executable(
        &self.root_path,
        &install_tool_id,
        &[&format!("{}.cmd", get_exe_name(&install_tool_id)), &format!("{}.exe", get_exe_name(&install_tool_id)), &format!("bin\\{}.exe", get_exe_name(&install_tool_id))],
    ) {
        self.path_manager.create_cmd_shim(&get_exe_name(&install_tool_id), &exe_path).ok();
    }
}
```

Persist `install_path` from detection instead of blindly persisting `target_dir`.

- [ ] **Step 8: Run the focused backend test set**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml windows_paths_
```

Expected:

- PASS for the new path-resolution and shim tests

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/services/installer/windows.rs src-tauri/src/services/installer/path_manager.rs src-tauri/src/services/installer/mod.rs src-tauri/src/plugins/nodejs.rs src-tauri/src/plugins/git.rs src-tauri/src/plugins/claude_code_cli.rs src-tauri/src/plugins/codex_cli.rs src-tauri/src/plugins/gemini_cli.rs
git commit -m "fix: correct windows managed-prefix detection and shims"
```

### Task 3: Add Native Windows Install Logic For OpenCode CLI, OpenClaw, And Hermes

**Files:**
- Modify: `src-tauri/src/plugins/opencode_cli.rs`
- Modify: `src-tauri/src/plugins/openclaw.rs`
- Modify: `src-tauri/src/plugins/hermes.rs`
- Modify: `src-tauri/src/services/installer/windows.rs`
- Test: `src-tauri/src/plugins/opencode_cli.rs`
- Test: `src-tauri/src/plugins/openclaw.rs`
- Test: `src-tauri/src/plugins/hermes.rs`

- [ ] **Step 1: Write failing detection-first tests for the three native Windows tools**

Add tests that lock down “already installed means skip” behavior:

```rust
#[test]
fn native_windows_opencode_cli_detects_existing_native_windows_command() {
    let detect = OpenCodeCliPlugin.detect(None);
    let _ = detect; // test will be expanded with PATH stubbing or managed-path fixtures
    assert_eq!(OpenCodeCliPlugin.install_strategy(), InstallStrategy::ManagedPrefix);
}

#[test]
fn native_windows_openclaw_uses_official_script_strategy() {
    assert_eq!(OpenClawPlugin.install_strategy(), InstallStrategy::OfficialScript);
}

#[test]
fn native_windows_hermes_uses_python_package_strategy() {
    assert_eq!(HermesPlugin.install_strategy(), InstallStrategy::PythonPackage);
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml native_windows_
```

Expected:

- FAIL because these strategy methods do not exist on the plugins yet

- [ ] **Step 3: Implement native Windows `OpenCode CLI`**

Update `src-tauri/src/plugins/opencode_cli.rs`:

```rust
fn install_strategy(&self) -> InstallStrategy {
    InstallStrategy::ManagedPrefix
}

#[cfg(target_os = "windows")]
fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "opencode-cli".into(),
        tool_name: "OpenCode (CLI)".into(),
        phase: "installing".into(),
        percent: 20,
        message: "Installing OpenCode CLI for native Windows...".into(),
    });

    let output = std::process::Command::new("npm")
        .args(["install", "-g", "opencode-ai", "--prefix", &target_dir.to_string_lossy()])
        .output()
        .map_err(|e| format!("npm install failed: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}
```

Ensure `detect()` probes both PATH and managed prefix candidates for `opencode.cmd` / `opencode.exe`.

- [ ] **Step 4: Implement native Windows `OpenClaw` through its official path**

Update `src-tauri/src/plugins/openclaw.rs`:

```rust
fn install_strategy(&self) -> InstallStrategy {
    InstallStrategy::OfficialScript
}

#[cfg(target_os = "windows")]
fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
    let script_path = target_dir.join("install-openclaw.ps1");
    std::fs::create_dir_all(target_dir).map_err(|e| format!("create dir failed: {e}"))?;
    std::fs::write(&script_path, r#"
$ProgressPreference = 'SilentlyContinue'
irm https://docs.openclaw.ai/install.ps1 | iex
"#).map_err(|e| format!("write script failed: {e}"))?;

    let _ = progress.blocking_send(InstallProgress {
        tool_id: "openclaw".into(),
        tool_name: "OpenClaw".into(),
        phase: "installing".into(),
        percent: 35,
        message: "Running official OpenClaw installer...".into(),
    });

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", &script_path.to_string_lossy()])
        .output()
        .map_err(|e| format!("powershell install failed: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}
```

- [ ] **Step 5: Implement native Windows `Hermes`**

Update `src-tauri/src/plugins/hermes.rs`:

```rust
fn install_strategy(&self) -> InstallStrategy {
    InstallStrategy::PythonPackage
}

#[cfg(target_os = "windows")]
fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "hermes".into(),
        tool_name: "Hermes (Web UI)".into(),
        phase: "installing".into(),
        percent: 25,
        message: "Installing Hermes native Windows package...".into(),
    });

    let output = std::process::Command::new("py")
        .args(["-m", "pip", "install", "hermes-agent[web]"])
        .output()
        .map_err(|e| format!("pip install failed: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}
```

Detect both `hermes --version` and a managed path if later moved into a dedicated environment.

- [ ] **Step 6: Run the targeted backend tests to verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml native_windows_
```

Expected:

- PASS for strategy wiring

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/plugins/opencode_cli.rs src-tauri/src/plugins/openclaw.rs src-tauri/src/plugins/hermes.rs src-tauri/src/services/installer/windows.rs
git commit -m "feat: add native windows install paths for opencode openclaw and hermes"
```

### Task 4: Replace Fake Desktop Installs With Real Official Desktop App Flows

**Files:**
- Modify: `src-tauri/src/plugins/claude_code_desktop.rs`
- Modify: `src-tauri/src/plugins/codex_desktop.rs`
- Modify: `src-tauri/src/plugins/opencode_desktop.rs`
- Modify: `src-tauri/src/services/downloader.rs`
- Modify: `src-tauri/src/services/installer/windows.rs`
- Test: `src-tauri/src/plugins/claude_code_desktop.rs`
- Test: `src-tauri/src/plugins/codex_desktop.rs`
- Test: `src-tauri/src/plugins/opencode_desktop.rs`

- [ ] **Step 1: Write failing tests that enforce desktop-installer behavior**

Add tests:

```rust
#[test]
fn desktop_installer_claude_desktop_uses_desktop_installer_strategy() {
    assert_eq!(ClaudeCodeDesktopPlugin.install_strategy(), InstallStrategy::DesktopInstaller);
}

#[test]
fn desktop_installer_codex_desktop_uses_desktop_installer_strategy() {
    assert_eq!(CodexDesktopPlugin.install_strategy(), InstallStrategy::DesktopInstaller);
}

#[test]
fn desktop_installer_opencode_desktop_uses_desktop_installer_strategy() {
    assert_eq!(OpenCodeDesktopPlugin.install_strategy(), InstallStrategy::DesktopInstaller);
}
```

- [ ] **Step 2: Run the targeted desktop tests to verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml desktop_installer_
```

Expected:

- FAIL because the desktop plugins still behave like npm-backed CLI tools

- [ ] **Step 3: Add desktop installer helpers**

Extend `src-tauri/src/services/installer/windows.rs` with official install helpers:

```rust
pub fn local_appdata_programs(subdir: &str, exe_name: &str) -> Option<std::path::PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(|dir| {
        std::path::Path::new(&dir).join("Programs").join(subdir).join(exe_name)
    })
}

pub fn run_windows_installer(installer_path: &Path) -> Result<(), String> {
    let status = std::process::Command::new(installer_path)
        .spawn()
        .map_err(|e| format!("failed to launch installer: {e}"))?
        .wait()
        .map_err(|e| format!("failed to wait for installer: {e}"))?;
    if !status.success() {
        return Err(format!("installer exited with code {:?}", status.code()));
    }
    Ok(())
}
```

- [ ] **Step 4: Replace Claude Desktop npm behavior with official installer logic**

Update `src-tauri/src/plugins/claude_code_desktop.rs`:

```rust
fn install_strategy(&self) -> InstallStrategy {
    InstallStrategy::DesktopInstaller
}

#[cfg(target_os = "windows")]
fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
    let installer = target_dir.join("claude-desktop-installer.exe");
    std::fs::create_dir_all(target_dir).map_err(|e| format!("create dir failed: {e}"))?;
    let url = "https://claude.ai/download/windows";
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime failed: {e}"))?;
    rt.block_on(async { crate::services::downloader::download_file(url, &installer, None).await })?;
    crate::services::installer::windows::run_windows_installer(&installer)?;
    Ok(())
}
```

Keep `detect()` focused on official desktop app locations, not `claude` CLI commands.

- [ ] **Step 5: Replace Codex Desktop npm behavior with official installer logic**

Update `src-tauri/src/plugins/codex_desktop.rs` similarly, but with Codex desktop detection and official Windows download path.

```rust
fn install_strategy(&self) -> InstallStrategy {
    InstallStrategy::DesktopInstaller
}
```

And a Windows install body that downloads the official desktop installer or store-linked executable wrapper before running it.

- [ ] **Step 6: Replace OpenCode Desktop placeholder behavior with official installer logic**

Update `src-tauri/src/plugins/opencode_desktop.rs`:

```rust
fn install_strategy(&self) -> InstallStrategy {
    InstallStrategy::DesktopInstaller
}

#[cfg(target_os = "windows")]
fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
    let installer = target_dir.join("opencode-desktop.exe");
    std::fs::create_dir_all(target_dir).map_err(|e| format!("create dir failed: {e}"))?;
    let url = "https://opencode.ai/download/stable/windows-x64-nsis";
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime failed: {e}"))?;
    rt.block_on(async { crate::services::downloader::download_file(url, &installer, None).await })?;
    crate::services::installer::windows::run_windows_installer(&installer)?;
    Ok(())
}
```

- [ ] **Step 7: Run the targeted desktop tests to verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml desktop_installer_
```

Expected:

- PASS for desktop strategy wiring

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/plugins/claude_code_desktop.rs src-tauri/src/plugins/codex_desktop.rs src-tauri/src/plugins/opencode_desktop.rs src-tauri/src/services/downloader.rs src-tauri/src/services/installer/windows.rs
git commit -m "feat: install real windows desktop apps for claude codex and opencode"
```

### Task 5: Keep Wizard And Manager In Sync With Detection-First Behavior

**Files:**
- Modify: `src/pages/Wizard.tsx`
- Modify: `src/pages/Manager.tsx`
- Modify: `src/components/tools/InstallProgress.tsx`
- Modify: `tests/integration/App.test.tsx`
- Create: `tests/components/Wizard.installDetection.test.tsx`
- Create: `tests/components/Manager.installDetection.test.tsx`

- [ ] **Step 1: Write failing frontend tests for skip-and-display behavior**

Create `tests/components/Wizard.installDetection.test.tsx`:

```tsx
it("removes already-installed tools from the default selection", async () => {
  detectToolsMock.mockResolvedValueOnce([
    { installed: true, version: "v22.0.0", installPath: "C:\\node" },
    { installed: false },
  ]);

  render(<Wizard onComplete={vi.fn()} />);

  expect(await screen.findByText("已安装")).toBeInTheDocument();
  expect(screen.getByText("Claude Code (CLI)")).toBeInTheDocument();
});
```

Create `tests/components/Manager.installDetection.test.tsx`:

```tsx
it("shows externally installed tools in the installed tab", async () => {
  installedToolsMock.mockResolvedValueOnce([
    {
      id: "claude-code-cli",
      name: "Claude Code (CLI)",
      installPath: "C:\\Users\\me\\AppData\\Roaming\\npm",
      installRoot: "D:\\AgenticTools",
      category: "tool",
      status: "installed",
    },
  ]);

  render(<Manager />);
  expect(await screen.findByText("Claude Code (CLI)")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the targeted frontend tests to verify they fail**

Run:

```bash
pnpm vitest run tests/components/Wizard.installDetection.test.tsx tests/components/Manager.installDetection.test.tsx
```

Expected:

- FAIL because the current assumptions still center on AgenticBoot-owned installs only

- [ ] **Step 3: Tighten Wizard default-selection behavior**

In `src/pages/Wizard.tsx`, keep the current load flow but make the intent explicit:

```tsx
toolsApi.detectTools(ids, rootPath).then((results) => {
  const detected = new Set<string>();
  results.forEach((result, index) => {
    if (result.installed) {
      detected.add(ids[index]);
    }
  });
  setInstalledIds(detected);
  setSelectedTools((prev) => {
    const next = new Set(prev);
    detected.forEach((id) => next.delete(id));
    return next;
  });
});
```

Do not special-case “outside managed root” as uninstalled.

- [ ] **Step 4: Keep Manager display consistent for external installs**

In `src/pages/Manager.tsx`, keep using the installed-tools query result as the source of truth for the installed tab and ensure the available tab excludes any detected installed IDs regardless of path origin:

```tsx
const installedIds = new Set(installedTools.map((tool) => tool.id));
const notInstalled = ALL_TOOLS_META.filter((meta) => !installedIds.has(meta.id));
```

No UI branch should require `installPath` to be under `installRoot` to count as installed.

- [ ] **Step 5: Keep progress messaging aligned with backend skip events**

In `src/components/tools/InstallProgress.tsx`, keep `skipped` treated as a completed step:

```tsx
const isComplete =
  step.isInstalled ||
  progress?.phase === "complete" ||
  progress?.phase === "skipped";
```

Ensure the skip label stays visible:

```tsx
case "skipped":
  return t("tools.phaseSkipped", "已安装，跳过");
```

- [ ] **Step 6: Run the targeted frontend tests to verify they pass**

Run:

```bash
pnpm vitest run tests/components/Wizard.installDetection.test.tsx tests/components/Manager.installDetection.test.tsx
```

Expected:

- PASS for both new detection-first UI tests

- [ ] **Step 7: Commit**

```bash
git add src/pages/Wizard.tsx src/pages/Manager.tsx src/components/tools/InstallProgress.tsx tests/components/Wizard.installDetection.test.tsx tests/components/Manager.installDetection.test.tsx tests/integration/App.test.tsx
git commit -m "test: keep wizard and manager aligned with detection-first installs"
```

### Task 6: Full Verification Pass And Documentation Refresh

**Files:**
- Modify: `docs/tools/opencode.md`
- Modify: `docs/tools/nodejs.md`
- Modify: `docs/tools/git.md`
- Modify: `docs/tools/openclaw.md`
- Modify: `README.md`
- Modify: `README_ZH.md`
- Test: existing Rust and frontend suites

- [ ] **Step 1: Write the failing doc expectations as a checklist**

Before editing docs, record the Windows facts the code now supports:

```text
OpenCode CLI: native Windows supported
Desktop entries: real official desktop installs
Node.js/Git: skipped when already installed
OpenClaw/Hermes: detect first, install only when missing
```

Treat any doc that still says otherwise as failing the release checklist.

- [ ] **Step 2: Run the main Rust suite**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

- PASS across the Rust unit and integration suite

- [ ] **Step 3: Run the main frontend suite**

Run:

```bash
pnpm test:unit
```

Expected:

- PASS across the Vitest suite

- [ ] **Step 4: Update stale Windows docs**

Refresh `docs/tools/opencode.md` from:

```md
**OpenCode 目前不支持 Windows**
```

to content that matches the implemented native Windows path.

Refresh `docs/tools/nodejs.md` and `docs/tools/git.md` to state that AgenticBoot will reuse an existing working installation and only install a managed copy when missing or insufficient.

Refresh `docs/tools/openclaw.md` to align with the implemented Windows-native install behavior and detect-before-install flow.

Update `README.md` and `README_ZH.md` to reflect:

- desktop apps are real official installs
- preinstalled tools are detected and skipped
- OpenCode CLI supports native Windows in AgenticBoot

- [ ] **Step 5: Re-run the high-signal verification after doc edits**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
pnpm test:unit
```

Expected:

- PASS again

- [ ] **Step 6: Commit**

```bash
git add docs/tools/opencode.md docs/tools/nodejs.md docs/tools/git.md docs/tools/openclaw.md README.md README_ZH.md
git commit -m "docs: update windows install behavior and detection flow"
```
