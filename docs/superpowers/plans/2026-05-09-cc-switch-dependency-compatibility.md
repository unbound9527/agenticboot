# CC Switch Dependency Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce dependency drift from upstream `CC Switch` without regressing the current desktop startup path.

**Architecture:** Keep the dependency surface intentionally narrow. Start with the Tauri family in `package.json` and `src-tauri/Cargo.toml`, then reconcile the generated lockfiles only as needed so the repo still boots with the existing dev script. If startup still fails, treat that as a runtime defect and stop expanding dependency changes.

**Tech Stack:** pnpm 9, Tauri 2, Cargo, PowerShell dev script, Vite.

---

### Task 1: Align dependency declarations with the compatibility baseline

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Update the Tauri package specifiers to the chosen compatibility shape**

```json
{
  "devDependencies": {
    "@tauri-apps/cli": "^2.8.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.8.0",
    "@tauri-apps/plugin-dialog": "^2.4.0",
    "@tauri-apps/plugin-process": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "@tauri-apps/plugin-updater": "^2.0.0"
  }
}
```

```toml
[build-dependencies]
tauri-build = { version = "2.4.0", features = [] }

[dependencies]
tauri = { version = "2.8.2", features = ["tray-icon", "protocol-asset", "image-png"] }
tauri-plugin-process = "2"
tauri-plugin-updater = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
```

- [ ] **Step 2: Keep every other dependency declaration unchanged**

The rest of the manifest should remain byte-for-byte the same so this pass stays focused and mergeable.

### Task 2: Reconcile lockfiles to the manifest changes

**Files:**
- Modify: `pnpm-lock.yaml`
- Modify only if Cargo resolution changes: `src-tauri/Cargo.lock`

- [ ] **Step 1: Regenerate the pnpm lockfile from the updated manifest**

Run: `pnpm install --lockfile-only`

Expected: `pnpm-lock.yaml` updates only for the Tauri package specifiers and any transitive entries required by those specifiers.

- [ ] **Step 2: Refresh the Rust lockfile only if Cargo reports a dependency mismatch**

Run: `cargo check --locked`

Expected: either a clean dependency resolution or a clear Cargo error that points to the exact package mismatch.

- [ ] **Step 3: Confirm the lockfile diff is narrow**

Run: `git diff -- pnpm-lock.yaml src-tauri/Cargo.lock`

Expected: only Tauri-related entries changed.

### Task 3: Verify the startup baseline still works

**Files:**
- No file edits expected

- [ ] **Step 1: Run the repository-managed desktop startup script**

Run: `.\scripts\dev-desktop.ps1`

Expected: Vite starts, Tauri launches, and the app window appears instead of exiting immediately.

- [ ] **Step 2: Confirm the process does not reproduce the previous immediate exit**

Expected: no `target\\debug\\agenticboot.exe` exit with `0xcfffffff` during startup.

### Task 4: Summarize the final compatibility state

**Files:**
- No file edits expected

- [ ] **Step 1: Record the exact dependency delta that remains from upstream**

Expected: a concise summary of which Tauri versions still differ, if any, and why they were kept.

- [ ] **Step 2: Call out any remaining runtime failure separately from dependency drift**

Expected: if startup still fails, classify it as a runtime issue, not a dependency mismatch.
