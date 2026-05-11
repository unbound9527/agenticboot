# Gemini CLI npm Registry Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Gemini CLI install use the official npm registry when npm connectivity is available, and automatically fall back to the npm mirror when official npm connectivity is unavailable.

**Architecture:** Compute npm reachability once inside the installer service, carry the chosen registry mode through `ToolInstallContext`, and let the Gemini CLI plugin append the correct npm install arguments. Keep the registry decision isolated from UI code so both Wizard and Manager installs behave the same way.

**Tech Stack:** Rust, Tauri, npm CLI, Vitest

---

### Task 1: Add a failing registry-selection test

**Files:**
- Modify: `src-tauri/src/plugins/npm_cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn npm_cli_install_uses_mirror_registry_when_requested() {
    let temp = tempfile::tempdir().unwrap();
    let target_dir = temp.path().join("gemini-cli");
    let (tx, mut rx) = tokio::sync::mpsc::channel::<InstallProgress>(4);

    install_npm_cli_with_registry_and_runner(
        &target_dir,
        temp.path(),
        "gemini-cli",
        "Gemini CLI",
        tx,
        "@google/gemini-cli",
        NpmRegistrySource::Mirror,
        |_install_root, args, context| {
            assert_eq!(context, "npm install failed");
            assert!(args.contains(&"--registry"));
            assert!(args.contains(&"https://registry.npmmirror.com"));
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(rx.try_recv().unwrap().phase, "downloading");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test npm_cli_install_uses_mirror_registry_when_requested -p agenticboot-tauri -q`
Expected: FAIL because `install_npm_cli_with_registry_and_runner` does not exist yet.

### Task 2: Implement Gemini registry selection

**Files:**
- Modify: `src-tauri/src/plugin.rs`
- Modify: `src-tauri/src/services/installer/mod.rs`
- Modify: `src-tauri/src/plugins/npm_cli.rs`
- Modify: `src-tauri/src/plugins/gemini_cli.rs`

- [ ] **Step 1: Add the registry mode to install context and compute it once per install plan**
- [ ] **Step 2: Add a Gemini-specific npm install helper that appends the mirror registry only when requested**
- [ ] **Step 3: Override Gemini CLI install-with-context to use the context-aware helper**
- [ ] **Step 4: Run the targeted Rust tests**

Run:
`cargo test npm_cli_install_uses_mirror_registry_when_requested gemini_cli::tests::native_windows_gemini_cli_detects_existing_managed_windows_command -p agenticboot-tauri -q`

Expected: PASS

### Task 3: Verify the end-to-end install path

**Files:**
- No additional code changes expected

- [ ] **Step 1: Run the relevant Rust test suite**

Run: `cargo test -p agenticboot-tauri -q`

- [ ] **Step 2: Run the focused frontend unit tests for install detection**

Run: `pnpm test:unit -- tests/components/Wizard.installDetection.test.tsx tests/components/Manager.installDetection.test.tsx`

Expected: PASS
