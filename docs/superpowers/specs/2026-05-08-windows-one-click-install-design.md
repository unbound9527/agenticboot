# Windows One-Click Install Design

> Scope: add reliable Windows one-click install logic for AgenticBoot's tool catalog, while keeping macOS and Linux as framework-only stubs for now.

## Goal

Implement a Windows-first installation system that can:

- Detect whether each supported tool is already installed before doing any work
- Install CLI tools into AgenticBoot-managed locations when appropriate
- Install desktop apps as real official desktop applications instead of aliasing them to CLI packages
- Avoid unsafe uninstall behavior for system-managed desktop installs
- Preserve the current CC Switch-derived plugin architecture instead of replacing it

## User Constraints

The approved behavior for this iteration is:

- `OpenCode CLI` must use native Windows installation and must not depend on WSL
- `Hermes Web UI` must support a native Windows path and must not be gated on WSL2
- Every tool must perform installed-state detection first so already-installed tools are skipped
- This detect-before-install rule applies to all supported tools, not just shared dependencies such as `nodejs` and `git`
- Windows implementations should be real and usable now
- macOS and Linux should get the same framework and error surfaces, but not real install commands yet

## Current State

The current codebase already has a good high-level shape:

- Frontend selection and progress UI live in `src/pages/Wizard.tsx`, `src/pages/Manager.tsx`, and `src/components/tools/InstallProgress.tsx`
- Tauri commands live in `src-tauri/src/commands/tools.rs`
- Install orchestration lives in `src-tauri/src/services/installer/mod.rs`
- Install root and shim behavior live in `src-tauri/src/services/installer/path_manager.rs`
- Tool-specific behavior lives behind `ToolPlugin` in `src-tauri/src/plugin.rs` and `src-tauri/src/plugins/*.rs`

The main problems are not lack of entry points, but incorrect assumptions:

1. The installer currently assumes all tools behave like a managed local prefix under `<root>/<tool-id>`.
2. The installer currently assumes all installs should produce local shims and be removable via directory deletion.
3. Desktop entries such as `claude-code-desktop` and `codex-desktop` currently install npm packages instead of official desktop apps.
4. Windows path detection for Node.js and npm-installed CLIs currently assumes Unix-like `bin/` layouts that do not consistently match Windows `--prefix` output.
5. `opencode-cli` on Windows is currently hardcoded as unsupported even though the intended product behavior is native Windows support.

## Supported Windows Targets

This design covers all items currently exposed by the Wizard / Manager catalog:

### Dependencies

- `nodejs`
- `git`

### CLI / agent tools

- `claude-code-cli`
- `codex-cli`
- `gemini-cli`
- `opencode-cli`
- `openclaw`
- `hermes`

### Desktop applications

- `claude-code-desktop`
- `codex-desktop`
- `opencode-desktop`

## Design Overview

The recommended approach is to keep the existing `ToolPlugin` model but add installation-strategy awareness to the backend. The install engine should stop assuming that every tool is a local-prefix tool.

Windows tools fall into four operational buckets:

1. `ManagedPrefix`
   Used for tools AgenticBoot installs into its own managed root and can safely shim and remove.
   Examples: `nodejs`, `git`, npm-based CLIs, and Windows-native `opencode-cli` if installed into a managed prefix.

2. `OfficialScript`
   Used when the upstream-supported install path is a vendor script that handles its own internal logic.
   Example: `openclaw`.

3. `PythonPackage`
   Used when the upstream-supported install path is a Python package command and the executable is provided by Python packaging rather than npm.
   Example: `hermes`.

4. `DesktopInstaller`
   Used for real desktop apps installed via official Windows installers or store/MSIX flows.
   Examples: `claude-code-desktop`, `codex-desktop`, `opencode-desktop`.

This classification is internal to the backend. The frontend can keep its current simple list and progress model.

## Detection Requirements

Detection must happen before installation for every selected tool and dependency. The install plan should mark already-installed tools as `is_installed = true`, and the execution phase should emit a `skipped` progress event instead of reinstalling them.

This rule applies uniformly to:

- Shared dependencies such as `nodejs` and `git`
- CLI tools such as `claude-code-cli`, `codex-cli`, `gemini-cli`, `opencode-cli`, `openclaw`, and `hermes`
- Desktop applications such as `claude-code-desktop`, `codex-desktop`, and `opencode-desktop`

Detection must combine:

1. Runtime detection
   Probe known commands, well-known install paths, and Windows desktop install locations.

2. AgenticBoot-managed install root detection
   Probe the configured install root for tools previously installed into the managed directory.

3. Database fallback
   Use `installed_tools` records when runtime probing does not immediately resolve the tool but AgenticBoot has a successful record.

Detection rules by category:

### ManagedPrefix tools

- Check command availability first
- Check managed install-root locations second
- Return version when practical

### DesktopInstaller tools

- Check known official Windows install paths
- Check registered install locations where feasible
- Never pretend a CLI npm package counts as the desktop app

### Global detection policy

- If a supported tool is already present and usable on the machine, AgenticBoot must skip reinstalling it even when the existing install was not created by AgenticBoot
- If a supported tool is already present but does not satisfy the required behavior or version floor, AgenticBoot may install its own managed copy as a fallback
- Reuse of an existing system install must never make uninstall unsafe; AgenticBoot may only remove installs it owns

### PythonPackage tools

- Check `hermes --version` first
- Check managed Python virtual environment or managed script location if AgenticBoot owns the install

## Windows Installation Strategy Per Tool

### Node.js

Keep Node.js as an AgenticBoot-managed dependency installed under the managed root. The current implementation can stay zip-based, but the path logic must be corrected for the real Windows layout so detection and shims work reliably.

Requirements:

- Fix post-extract layout handling
- Detect the actual executable location for Windows
- Provide a stable way for downstream npm-based installs to use the managed Node runtime

### Git

Keep Git as an AgenticBoot-managed dependency installed under the managed root using the current MinGit-style zip approach, as this avoids `winget` latency and store dependency.

Requirements:

- Keep installation self-contained
- Detect the actual Windows executable layout
- Preserve safe uninstall by deleting only the managed Git directory

### Claude Code CLI / Codex CLI / Gemini CLI

Keep these as npm-based CLI installs into the managed root, but correct all Windows assumptions.

Requirements:

- Install with npm into the managed tool directory
- Detect the actual executable location produced by Windows npm prefix installs
- Generate working shims that invoke the managed runtime correctly
- Skip install if the command already exists and satisfies detection

### OpenCode CLI

Implement a native Windows path and do not use WSL.

Requirements:

- Support native Windows installation
- Prefer a managed installation path controlled by AgenticBoot
- Integrate with the same detect-before-install flow as the other CLIs
- Do not expose or require WSL for the one-click flow
- Skip installation when a native Windows `opencode` command is already present and usable

### OpenClaw

Use the upstream-supported Windows installation method instead of forcing a homemade npm-only approximation.

Requirements:

- Support native Windows install via PowerShell-friendly official path
- Detect existing `openclaw` command before install
- Keep configuration paths consistent with existing `openclaw_config.rs`
- Treat it as installed only when the command or managed install is actually available

### Hermes Web UI

Implement a native Windows path and do not require WSL2.

Requirements:

- Detect existing `hermes` command before install
- Install Hermes using a native Windows-compatible path
- Ensure the resulting install supports `hermes dashboard`
- Keep compatibility with the existing Hermes config and launcher code in `src-tauri/src/commands/hermes.rs`

### Claude Code Desktop

Install the real official Windows desktop application.

Requirements:

- Stop installing the npm CLI package for the desktop entry
- Detect the official app location, not the CLI command
- Store installation metadata as a system desktop install, not as a managed-prefix CLI
- Use a desktop-installer flow instead of local shims
- Skip installation when the official Windows desktop app is already installed

### Codex Desktop

Install the real official Windows desktop application.

Requirements:

- Stop mapping this entry to the CLI package
- Detect official Windows desktop presence
- Install through a desktop-installer flow
- Treat uninstall separately from managed-prefix tools
- Skip installation when the official Windows desktop app is already installed

### OpenCode Desktop

Install the real official Windows desktop application.

Requirements:

- Use a desktop-installer flow
- Detect official Windows install presence
- Keep it separate from `opencode-cli`
- Skip installation when the official Windows desktop app is already installed

## Backend Changes

### 1. Extend plugin metadata

Update the backend plugin model so each tool declares its install behavior explicitly.

Recommended addition:

- `install_strategy`
- `detection_hints`
- `managed_by_root`

The exact Rust shape can be an enum plus helper methods, for example:

- `ManagedPrefix`
- `OfficialScript`
- `PythonPackage`
- `DesktopInstaller`

This metadata should be returned by each plugin implementation rather than inferred from the tool ID.

### 2. Split install execution by strategy

`InstallerService::execute_install_plan` should branch on the plugin strategy instead of always doing the same post-install work.

Behavior by strategy:

- `ManagedPrefix`
  Create managed install dir, run installer, create shims if needed, record managed path

- `OfficialScript`
  Run the official script or command path, record detected result afterward, do not assume local shims

- `PythonPackage`
  Create or reuse a managed Python install location, install package, record executable path, avoid npm assumptions

- `DesktopInstaller`
  Download or launch the official desktop installer, re-detect afterward, store detected install path or package id, do not create CLI shims

### 3. Fix post-install detection

After any install completes, the service should re-run plugin detection and persist the detected path and version from the actual environment instead of guessing them from `target_dir`.

### 4. Make uninstall strategy-aware

Current uninstall behavior is too dangerous for desktop/system installs because it always tends toward deleting directories.

New rule:

- `ManagedPrefix`
  Safe to remove local shim and managed directory

- `OfficialScript`
  Use plugin uninstall logic only; do not generically delete arbitrary install directories unless the plugin explicitly marks them as AgenticBoot-owned

- `PythonPackage`
  Remove only AgenticBoot-managed environment artifacts if Hermes is installed into a managed environment

- `DesktopInstaller`
  Use desktop-specific uninstall logic only; never call generic `remove_dir_all(record.install_path)` for system install locations

### 5. Improve path/shim behavior

`PathManager` currently assumes all CLI shims should live under `<root>/bin` and may even assume npm itself lives next to those shims. The new logic should instead generate shims only for tools that truly need them and should point them at the actual managed executable path.

This means:

- General CLI shim generation should be path-based, not package-name-based
- npm-specific shims should only be used if the managed install layout actually requires them
- desktop apps should not get CLI shims

## Database Behavior

`installed_tools` remains the source of truth for AgenticBoot history, but the meaning of `install_path` needs to become strategy-aware.

Expected semantics:

- `ManagedPrefix`
  `install_path` is the AgenticBoot-managed directory

- `PythonPackage`
  `install_path` is the managed venv or managed script root

- `DesktopInstaller`
  `install_path` is the detected desktop install location when available, or a stable package identifier when the installer is store-managed

The `category` field can remain unchanged for now. The install strategy should live in code rather than requiring an immediate schema change.

## Frontend Behavior

The current frontend flow is mostly adequate and should remain simple.

### Wizard

Keep:

- Tool selection
- Detect-on-load behavior
- Install root input
- Live install progress UI

Adjust:

- Already-installed tools must remain unselected by default and clearly marked as installed
- If a tool is installed outside the managed root, the UI should still treat it as installed and skip it

### Manager

Keep:

- Installed vs available tabs
- Uninstall actions
- Install-more entry point

Adjust:

- Display detected installed tools even when their actual install location is not inside the AgenticBoot root
- Ensure uninstall is only offered when the backend has a safe uninstall path for that strategy

## macOS and Linux Framework-Only Work

macOS and Linux should not gain real install logic in this iteration, but the strategy architecture should still be wired for them so future support does not require another refactor.

For non-Windows platforms:

- Keep plugin strategy metadata available
- Return clear “not implemented on this platform yet” errors
- Keep detection code where harmless, especially for already-installed tools
- Do not add real installer command execution yet

## Testing Requirements

Tests should cover the behavior that is currently fragile:

### Rust unit tests

- Windows executable path resolution for managed tools
- Strategy-aware uninstall safeguards
- Detect-before-install plan resolution
- Re-detection after install result persistence
- Desktop tools not producing managed CLI shims

### UI / hook tests

- Wizard excludes already-detected installed tools from default selection
- Manager shows installed tools even when install paths are outside the managed root
- Progress UI marks preinstalled tools as skipped

## Risks

1. Windows upstream installer flows may be slower or more stateful than npm installs.
2. Desktop installers may require more careful waiting and process-exit handling than the current synchronous plugin code assumes.
3. Hermes native Windows packaging may differ from npm-based tools and will require a path model that is not npm-centric.
4. Existing local docs for some tools are stale, so code should follow verified runtime behavior rather than old markdown assumptions.

## Out of Scope

- Real macOS install commands
- Real Linux install commands
- Reworking the frontend into a multi-step per-tool installer
- A full schema migration just to encode install strategy
- Provider/API onboarding for each installed tool

## Recommended Implementation Order

1. Add install-strategy support to plugin metadata and installer branching
2. Fix managed Windows path handling for Node.js, Git, and npm-based CLIs
3. Implement native Windows `OpenCode CLI`
4. Implement native Windows `OpenClaw`
5. Implement native Windows `Hermes`
6. Replace fake desktop npm installs with real desktop install flows
7. Harden uninstall logic and add regression tests
