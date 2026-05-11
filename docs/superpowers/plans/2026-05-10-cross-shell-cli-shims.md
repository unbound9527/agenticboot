# Cross-Shell CLI Shim Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make AgenticBoot-managed Windows CLI tools launch by bare command name in PowerShell, `cmd`, and Git Bash.

**Architecture:** Keep the existing managed-prefix install model and extend the shared `bin` shim generation so Windows installs publish both a `.cmd` launcher for native Windows shells and an extensionless shell script for Bash-compatible environments. Remove both artifacts during uninstall to keep managed cleanup symmetric.

**Tech Stack:** Rust, Tauri 2, winreg, tempfile, cargo test

---

## File Map

- Modify: `src-tauri/src/services/installer/path_manager.rs`
  Add shared Windows shim creation/removal that emits both `.cmd` and extensionless launchers into the managed `bin` directory.

- Modify: `src-tauri/src/services/installer/mod.rs`
  Switch managed install completion from single `.cmd` shim creation to the new cross-shell shim helper.

- Test: `src-tauri/src/services/installer/path_manager.rs`
  Lock down dual shim generation and symmetric cleanup.

### Task 1: Add Failing Shim Tests

**Files:**
- Modify: `src-tauri/src/services/installer/path_manager.rs`
- Test: `src-tauri/src/services/installer/path_manager.rs`

- [ ] **Step 1: Write the failing test**

Add a test that expects `create_windows_cli_shims("gemini", Path::new("D:\\AgenticTools\\gemini-cli\\gemini.cmd"))` to create:

```rust
tmp.path().join("bin").join("gemini.cmd")
tmp.path().join("bin").join("gemini")
```

and verify the extensionless shim contains a `#!/bin/sh` header plus the managed executable path.

- [ ] **Step 2: Run the focused Rust test and confirm it fails**

Run:

```powershell
cargo test windows_paths_cli_shim_targets_actual_managed_executable_path
```

Expected: FAIL because only `gemini.cmd` is currently created.

### Task 2: Implement Cross-Shell Shim Publishing

**Files:**
- Modify: `src-tauri/src/services/installer/path_manager.rs`
- Modify: `src-tauri/src/services/installer/mod.rs`

- [ ] **Step 1: Write the minimal implementation**

Add a new helper that writes:

```text
bin\<name>.cmd
bin\<name>
```

Use the managed executable path as the target in both launchers.

- [ ] **Step 2: Update install completion to call the new helper**

Replace the single `create_cmd_shim(...)` call with the new cross-shell shim helper for managed-prefix and Python-package tools.

### Task 3: Verify Cleanup Symmetry

**Files:**
- Modify: `src-tauri/src/services/installer/path_manager.rs`
- Test: `src-tauri/src/services/installer/path_manager.rs`

- [ ] **Step 1: Extend shim removal**

Delete both:

```text
bin\<name>.cmd
bin\<name>
```

- [ ] **Step 2: Run the focused Rust tests**

Run:

```powershell
cargo test path_manager
```

Expected: PASS.
