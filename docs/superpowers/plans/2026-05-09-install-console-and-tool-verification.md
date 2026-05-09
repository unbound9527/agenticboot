# Install Console And Tool Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a reusable install console that shows real install commands and logs for any tool, then wire and validate it one tool at a time for Claude Code (Desktop), Codex (Desktop), Gemini CLI, OpenCode (CLI), and OpenClaw.

**Architecture:** Keep the existing `install-progress` channel for compact progress summaries and add a new structured `install-log` channel for commands and raw output. Store the latest install session per tool in the frontend, render it with reusable UI, and route all backend tool installers through shared log helpers so new tools only need to emit standardized events.

**Tech Stack:** Tauri 2, Rust, React 18, TypeScript, React Query, Vitest, Radix UI, PowerShell, Cargo tests

---

## File Structure

### Backend

- Modify: `src-tauri/src/tool_types.rs`
  - Add serializable install log event types shared across commands and plugins.
- Modify: `src-tauri/src/services/installer/mod.rs`
  - Create session ids, emit install-log lifecycle events, and persist the existing install-progress flow.
- Create: `src-tauri/src/services/installer/logging.rs`
  - Central helper for session start, phase, command, output, and result emission.
- Modify: `src-tauri/src/services/installer/windows.rs`
  - Add command wrappers that capture stdout/stderr and forward log events.
- Modify: `src-tauri/src/services/mod.rs`
  - Export the new installer logging module through the existing service module tree.
- Modify: `src-tauri/src/plugins/claude_code_desktop.rs`
  - Route winget and fallback installer actions through shared install log helpers.
- Modify: `src-tauri/src/plugins/codex_desktop.rs`
  - Route Microsoft Store install logging through shared helpers.
- Modify: `src-tauri/src/plugins/gemini_cli.rs`
  - Route npm install logging through shared helpers.
- Modify: `src-tauri/src/plugins/opencode_cli.rs`
  - Route npm install logging through shared helpers.
- Modify: `src-tauri/src/plugins/openclaw.rs`
  - Keep staged progress and add install-log command/result visibility for the official script path.

### Frontend

- Modify: `src/types/tools.ts`
  - Mirror the new install log event and retained session types.
- Modify: `src/lib/api/tools.ts`
  - Add an `onInstallLog` listener.
- Create: `src/hooks/useInstallSessions.ts`
  - Store and update the latest install session per tool from `install-log` events.
- Create: `src/components/tools/InstallConsole.tsx`
  - Render summary and raw terminal-style output for one tool session.
- Modify: `src/components/tools/ToolCard.tsx`
  - Keep summary progress and expose an entrypoint to the retained install console.
- Modify: `src/components/tools/InstallProgress.tsx`
  - Show summary plus the live install console during multi-step installs.
- Modify: `src/pages/Manager.tsx`
  - Hold selected tool console state and surface retained logs after install completion.

### Tests And Verification

- Create: `tests/lib/installSessions.test.ts`
  - Verify retained latest-session behavior and event-to-state mapping.
- Modify: `tests/components/Manager.installDetection.test.tsx`
  - Verify install summary, live logs, retained logs, and console toggling.
- Modify: `tests/components/Wizard.installDetection.test.tsx`
  - Verify wizard does not regress while install event listeners expand.
- Modify: Rust tests in tool plugin files and installer helpers
  - Verify command and phase emission for each wired tool.
- Create: `scripts/test-claude-code-desktop-install.ps1`
- Create: `scripts/test-codex-desktop-install.ps1`
- Create: `scripts/test-gemini-cli-install.ps1`
- Create: `scripts/test-opencode-cli-install.ps1`
- Create: `scripts/test-openclaw-install.ps1`
  - Run isolated install flows and print the exact action taken plus success or failure.

## Task 1: Add Shared Install Log Types And Frontend API Plumbing

**Files:**
- Modify: `src/types/tools.ts`
- Modify: `src/lib/api/tools.ts`
- Create: `src/hooks/useInstallSessions.ts`
- Create: `tests/lib/installSessions.test.ts`

- [ ] **Step 1: Write the failing test for retained session typing and event handling**

```ts
import { describe, expect, it } from "vitest";
import { reduceInstallLogEvent } from "@/hooks/useInstallSessions";

describe("install session reducer", () => {
  it("creates a retained session from session-started and result events", () => {
    const sessionStarted = {
      toolId: "codex-desktop",
      toolName: "Codex (Desktop)",
      sessionId: "session-1",
      timestamp: "2026-05-09T12:00:00.000Z",
      level: "info" as const,
      kind: "session-started" as const,
      line: "Install session started",
    };

    const result = {
      ...sessionStarted,
      kind: "result" as const,
      level: "success" as const,
      line: "Install completed",
      exitCode: 0,
    };

    const started = reduceInstallLogEvent(new Map(), sessionStarted);
    const completed = reduceInstallLogEvent(started, result);

    expect(completed.get("codex-desktop")?.status).toBe("complete");
    expect(completed.get("codex-desktop")?.entries).toHaveLength(2);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node node_modules/vitest/vitest.mjs run tests/lib/installSessions.test.ts`

Expected: FAIL because `useInstallSessions`, `reduceInstallLogEvent`, and the install log types do not exist yet.

- [ ] **Step 3: Add the shared TypeScript types, listener contract, and session reducer**

```ts
export interface InstallLogEvent {
  toolId: string;
  toolName: string;
  sessionId: string;
  timestamp: string;
  phase?: string;
  level: "info" | "stdout" | "stderr" | "success" | "error";
  kind: "session-started" | "phase" | "command" | "output" | "result";
  line: string;
  command?: string;
  exitCode?: number | null;
}

export interface ToolInstallSession {
  toolId: string;
  toolName: string;
  sessionId: string;
  status: "running" | "complete" | "error";
  startedAt: string;
  endedAt?: string;
  lastSummary?: string;
  installPath?: string;
  entries: InstallLogEvent[];
}
```

```ts
onInstallLog(callback: (event: InstallLogEvent) => void): Promise<UnlistenFn> {
  return listen<InstallLogEvent>("install-log", (event) => {
    callback(event.payload);
  });
}
```

```ts
export function reduceInstallLogEvent(
  previous: Map<string, ToolInstallSession>,
  event: InstallLogEvent,
): Map<string, ToolInstallSession> {
  const next = new Map(previous);
  const current = next.get(event.toolId);

  const base: ToolInstallSession =
    !current || current.sessionId !== event.sessionId
      ? {
          toolId: event.toolId,
          toolName: event.toolName,
          sessionId: event.sessionId,
          status: "running",
          startedAt: event.timestamp,
          entries: [],
        }
      : current;

  next.set(event.toolId, {
    ...base,
    status:
      event.kind === "result" && event.level === "success"
        ? "complete"
        : event.kind === "result" && event.level === "error"
          ? "error"
          : base.status,
    endedAt: event.kind === "result" ? event.timestamp : base.endedAt,
    lastSummary: event.line,
    entries: [...base.entries, event],
  });

  return next;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `node node_modules/vitest/vitest.mjs run tests/lib/installSessions.test.ts`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/types/tools.ts src/lib/api/tools.ts src/hooks/useInstallSessions.ts tests/lib/installSessions.test.ts
git commit -m "feat: add install log event types"
```

## Task 2: Add Frontend Retained Session State And Reusable Install Console UI

**Files:**
- Create: `src/components/tools/InstallConsole.tsx`
- Modify: `src/components/tools/ToolCard.tsx`
- Modify: `src/components/tools/InstallProgress.tsx`
- Modify: `src/pages/Manager.tsx`
- Modify: `tests/components/Manager.installDetection.test.tsx`

- [ ] **Step 1: Write the failing component test for live and retained install logs**

```tsx
it("shows the latest retained install session for a tool after progress completes", async () => {
  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <Manager />
    </QueryClientProvider>,
  );

  await screen.findByText("Codex (CLI)");

  act(() => {
    installProgressListener?.({
      toolId: "codex-cli",
      toolName: "Codex (CLI)",
      phase: "installing",
      percent: 25,
      message: "Running npm install",
    });

    installLogListener?.({
      toolId: "codex-cli",
      toolName: "Codex (CLI)",
      sessionId: "session-1",
      timestamp: "2026-05-09T12:00:00.000Z",
      level: "info",
      kind: "command",
      line: "npm install -g @openai/codex --prefix D:\\AITools\\codex-cli",
      command: "npm install -g @openai/codex --prefix D:\\AITools\\codex-cli",
    });
  });

  expect(await screen.findByText(/npm install -g @openai\/codex/i)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx`

Expected: FAIL because `onInstallLog`, retained session state, and the install console UI are not wired.

- [ ] **Step 3: Implement retained session state and terminal-style console UI**

```tsx
export function InstallConsole({ session }: { session: ToolInstallSession | null }) {
  if (!session) return null;

  return (
    <div className="rounded-lg border bg-background">
      <div className="border-b px-4 py-2 font-mono text-xs">Install Console</div>
      <Tabs defaultValue="summary" className="w-full">
        <TabsList className="mx-4 mt-3">
          <TabsTrigger value="summary">Summary</TabsTrigger>
          <TabsTrigger value="raw">Raw Output</TabsTrigger>
        </TabsList>
        <TabsContent value="summary" className="px-4 py-3 text-sm">
          {session.entries.filter((entry) => entry.kind !== "output").map((entry) => (
            <div key={`${entry.timestamp}-${entry.line}`} className="font-mono text-xs">
              {entry.line}
            </div>
          ))}
        </TabsContent>
        <TabsContent value="raw" className="px-4 pb-4">
          <ScrollArea className="h-48 rounded bg-muted/30 p-3 font-mono text-xs">
            {session.entries.map((entry) => (
              <div key={`${entry.timestamp}-${entry.line}`}>{entry.line}</div>
            ))}
          </ScrollArea>
        </TabsContent>
      </Tabs>
    </div>
  );
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx tests/lib/installSessions.test.ts`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/hooks/useInstallSessions.ts src/components/tools/InstallConsole.tsx src/components/tools/ToolCard.tsx src/components/tools/InstallProgress.tsx src/pages/Manager.tsx tests/components/Manager.installDetection.test.tsx tests/lib/installSessions.test.ts
git commit -m "feat: add retained install console ui"
```

## Task 3: Add Shared Rust Install Log Types And Emission Helpers

**Files:**
- Modify: `src-tauri/src/tool_types.rs`
- Create: `src-tauri/src/services/installer/logging.rs`
- Modify: `src-tauri/src/services/installer/mod.rs`
- Modify: `src-tauri/src/services/installer/windows.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: Write the failing Rust test for install-log emission**

```rust
#[test]
fn install_log_helper_marks_result_with_exit_code() {
    let event = InstallLogEvent::result(
        "codex-desktop",
        "Codex (Desktop)",
        "session-1",
        "installing",
        "winget install completed",
        Some(0),
        true,
    );

    assert_eq!(event.kind, "result");
    assert_eq!(event.exit_code, Some(0));
    assert_eq!(event.level, "success");
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib install_log_helper_marks_result_with_exit_code`

Expected: FAIL because `InstallLogEvent::result` does not exist yet.

- [ ] **Step 3: Implement the serializable Rust types and emitter helpers**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallLogEvent {
    pub tool_id: String,
    pub tool_name: String,
    pub session_id: String,
    pub timestamp: String,
    pub phase: Option<String>,
    pub level: String,
    pub kind: String,
    pub line: String,
    pub command: Option<String>,
    pub exit_code: Option<i32>,
}
```

```rust
pub fn emit_command(
    app_handle: &AppHandle,
    tool_id: &str,
    tool_name: &str,
    session_id: &str,
    phase: &str,
    command: &str,
) {
    let _ = app_handle.emit(
        "install-log",
        InstallLogEvent::command(tool_id, tool_name, session_id, phase, command),
    );
}
```

- [ ] **Step 4: Run the Rust test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib install_log_helper_marks_result_with_exit_code`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/tool_types.rs src-tauri/src/services/installer/logging.rs src-tauri/src/services/installer/mod.rs src-tauri/src/services/installer/windows.rs src-tauri/src/services/mod.rs
git commit -m "feat: add shared install log emitters"
```

## Task 4: Wire Claude Code (Desktop) With Shared Install Logs And A Standalone Verification Script

**Files:**
- Modify: `src-tauri/src/plugins/claude_code_desktop.rs`
- Create: `scripts/test-claude-code-desktop-install.ps1`

- [ ] **Step 1: Write the failing Rust test for Claude desktop command visibility**

```rust
#[test]
fn claude_desktop_logs_winget_command_before_install() {
    let lines = build_claude_desktop_install_preview();

    assert!(lines.iter().any(|line| line.contains("winget install --id Anthropic.Claude")));
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib claude_desktop_logs_winget_command_before_install`

Expected: FAIL because the preview or shared logging hook does not exist yet.

- [ ] **Step 3: Implement command logging and the isolated verification script**

```rust
emit_phase(&progress, "installing", 10, "Trying winget install for Claude desktop");
emit_command(
    app_handle,
    "claude-code-desktop",
    "Claude Code (Desktop)",
    session_id,
    "installing",
    "winget install --id Anthropic.Claude -e --accept-package-agreements --accept-source-agreements",
);
```

```powershell
param(
  [string]$InstallRoot = "$env:LOCALAPPDATA\\AgenticBoot"
)

Write-Host "[phase] install"
Write-Host "[command] winget install --id Anthropic.Claude -e --accept-package-agreements --accept-source-agreements"
winget install --id Anthropic.Claude -e --accept-package-agreements --accept-source-agreements
if ($LASTEXITCODE -ne 0) {
  throw "Claude desktop install failed with exit code $LASTEXITCODE"
}
```

- [ ] **Step 4: Run the tests and script**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib claude_desktop_logs_winget_command_before_install`

Run: `powershell -ExecutionPolicy Bypass -File scripts/test-claude-code-desktop-install.ps1`

Expected: Rust test PASS, script prints the exact command before launching the real install path.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/plugins/claude_code_desktop.rs scripts/test-claude-code-desktop-install.ps1
git commit -m "feat: log claude desktop install commands"
```

## Task 5: Wire Codex (Desktop) With Shared Install Logs And A Standalone Verification Script

**Files:**
- Modify: `src-tauri/src/plugins/codex_desktop.rs`
- Create: `scripts/test-codex-desktop-install.ps1`

- [ ] **Step 1: Write the failing Rust test for Codex desktop command visibility**

```rust
#[test]
fn codex_desktop_logs_store_command_before_install() {
    let lines = build_codex_desktop_install_preview();

    assert!(lines.iter().any(|line| line.contains("winget install Codex -s msstore")));
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib codex_desktop_logs_store_command_before_install`

Expected: FAIL because the preview or logging hook does not exist yet.

- [ ] **Step 3: Implement command logging and the isolated verification script**

```rust
emit_phase(&progress, "installing", 10, "Launching Codex desktop through Microsoft Store");
emit_command(
    app_handle,
    "codex-desktop",
    "Codex (Desktop)",
    session_id,
    "installing",
    "winget install Codex -s msstore --accept-package-agreements --accept-source-agreements",
);
```

```powershell
Write-Host "[phase] install"
Write-Host "[command] winget install Codex -s msstore --accept-package-agreements --accept-source-agreements"
winget install Codex -s msstore --accept-package-agreements --accept-source-agreements
if ($LASTEXITCODE -ne 0) {
  throw "Codex desktop install failed with exit code $LASTEXITCODE"
}
```

- [ ] **Step 4: Run the tests and script**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib codex_desktop_logs_store_command_before_install`

Run: `powershell -ExecutionPolicy Bypass -File scripts/test-codex-desktop-install.ps1`

Expected: Rust test PASS, script prints the store command before launch.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/plugins/codex_desktop.rs scripts/test-codex-desktop-install.ps1
git commit -m "feat: log codex desktop install commands"
```

## Task 6: Wire Gemini CLI With Shared Install Logs And A Standalone Verification Script

**Files:**
- Modify: `src-tauri/src/plugins/gemini_cli.rs`
- Create: `scripts/test-gemini-cli-install.ps1`

- [ ] **Step 1: Write the failing Rust test for Gemini npm command logging**

```rust
#[test]
fn gemini_cli_logs_npm_command_before_install() {
    let lines = build_gemini_cli_install_preview("D:\\AITools\\gemini-cli");

    assert!(lines.iter().any(|line| line.contains("npm install -g @google/gemini-cli --prefix D:\\AITools\\gemini-cli")));
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_cli_logs_npm_command_before_install`

Expected: FAIL because Gemini install does not emit a preview or command log yet.

- [ ] **Step 3: Implement command logging and the isolated verification script**

```rust
run_npm_command_checked_with_logs(
    install_root,
    &[
        "install",
        "-g",
        "@google/gemini-cli",
        "--prefix",
        &target_dir.to_string_lossy(),
    ],
    "gemini-cli",
    "Gemini CLI",
    session_id,
)
```

```powershell
param(
  [string]$InstallRoot = "D:\\AITools",
  [string]$TargetDir = "D:\\AITools\\gemini-cli"
)

Write-Host "[phase] install"
Write-Host "[command] npm install -g @google/gemini-cli --prefix $TargetDir"
npm install -g @google/gemini-cli --prefix $TargetDir
if ($LASTEXITCODE -ne 0) {
  throw "Gemini CLI install failed with exit code $LASTEXITCODE"
}
```

- [ ] **Step 4: Run the tests and script**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_cli_logs_npm_command_before_install`

Run: `powershell -ExecutionPolicy Bypass -File scripts/test-gemini-cli-install.ps1`

Expected: Rust test PASS, script prints the npm command and exit result.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/plugins/gemini_cli.rs scripts/test-gemini-cli-install.ps1
git commit -m "feat: log gemini cli install commands"
```

## Task 7: Wire OpenCode (CLI) With Shared Install Logs And A Standalone Verification Script

**Files:**
- Modify: `src-tauri/src/plugins/opencode_cli.rs`
- Create: `scripts/test-opencode-cli-install.ps1`

- [ ] **Step 1: Write the failing Rust test for OpenCode npm command logging**

```rust
#[test]
fn opencode_cli_logs_npm_command_before_install() {
    let lines = build_opencode_cli_install_preview("D:\\AITools\\opencode-cli");

    assert!(lines.iter().any(|line| line.contains("npm install -g opencode-ai --prefix D:\\AITools\\opencode-cli")));
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib opencode_cli_logs_npm_command_before_install`

Expected: FAIL because OpenCode install does not emit a preview or command log yet.

- [ ] **Step 3: Implement command logging and the isolated verification script**

```rust
run_npm_command_checked_with_logs(
    install_root,
    &[
        "install",
        "-g",
        "opencode-ai",
        "--prefix",
        &target_dir.to_string_lossy(),
    ],
    "opencode-cli",
    "OpenCode (CLI)",
    session_id,
)
```

```powershell
param(
  [string]$InstallRoot = "D:\\AITools",
  [string]$TargetDir = "D:\\AITools\\opencode-cli"
)

Write-Host "[phase] install"
Write-Host "[command] npm install -g opencode-ai --prefix $TargetDir"
npm install -g opencode-ai --prefix $TargetDir
if ($LASTEXITCODE -ne 0) {
  throw "OpenCode CLI install failed with exit code $LASTEXITCODE"
}
```

- [ ] **Step 4: Run the tests and script**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib opencode_cli_logs_npm_command_before_install`

Run: `powershell -ExecutionPolicy Bypass -File scripts/test-opencode-cli-install.ps1`

Expected: Rust test PASS, script prints the npm command and exit result.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/plugins/opencode_cli.rs scripts/test-opencode-cli-install.ps1
git commit -m "feat: log opencode cli install commands"
```

## Task 8: Wire OpenClaw With Shared Install Logs And A Standalone Verification Script

**Files:**
- Modify: `src-tauri/src/plugins/openclaw.rs`
- Create: `scripts/test-openclaw-install.ps1`

- [ ] **Step 1: Write the failing Rust test for OpenClaw official script logging**

```rust
#[test]
fn openclaw_logs_official_powershell_command_before_waiting() {
    let lines = build_openclaw_install_preview();

    assert!(lines.iter().any(|line| line.contains("Invoke-RestMethod https://openclaw.ai/install.ps1")));
    assert!(lines.iter().any(|line| line.contains("Waiting for the official OpenClaw installer to finish")));
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib openclaw_logs_official_powershell_command_before_waiting`

Expected: FAIL because OpenClaw only emits staged progress and not a retained command preview yet.

- [ ] **Step 3: Implement command logging and the isolated verification script**

```rust
emit_command(
    app_handle,
    "openclaw",
    "OpenClaw",
    session_id,
    "installing",
    "powershell -NoProfile -ExecutionPolicy Bypass -Command & ([scriptblock]::Create((Invoke-RestMethod https://openclaw.ai/install.ps1))) -NoOnboard",
);
```

```powershell
Write-Host "[phase] install"
Write-Host "[command] powershell -NoProfile -ExecutionPolicy Bypass -Command & ([scriptblock]::Create((Invoke-RestMethod https://openclaw.ai/install.ps1))) -NoOnboard"
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ([scriptblock]::Create((Invoke-RestMethod https://openclaw.ai/install.ps1))) -NoOnboard"
if ($LASTEXITCODE -ne 0) {
  throw "OpenClaw install failed with exit code $LASTEXITCODE"
}
```

- [ ] **Step 4: Run the tests and script**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib openclaw_logs_official_powershell_command_before_waiting`

Run: `powershell -ExecutionPolicy Bypass -File scripts/test-openclaw-install.ps1`

Expected: Rust test PASS, script prints the official PowerShell action and preserves the waiting phase.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/plugins/openclaw.rs scripts/test-openclaw-install.ps1
git commit -m "feat: log openclaw install commands"
```

## Task 9: Run Cross-Stack Regression Tests And Manual Verification Sweep

**Files:**
- Modify: `tests/components/Manager.installDetection.test.tsx`
- Modify: `tests/components/Wizard.installDetection.test.tsx`
- Modify: Rust tests touched in Tasks 3-8

- [ ] **Step 1: Add the final failing assertions for summary and raw-view regressions**

```tsx
it("keeps summary progress visible while allowing raw output expansion", async () => {
  expect(await screen.findByText("Running npm install")).toBeInTheDocument();
  expect(screen.getByRole("tab", { name: "Raw Output" })).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the focused frontend regression suite**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx tests/lib/installSessions.test.ts`

Expected: PASS after the new assertions land.

- [ ] **Step 3: Run the focused backend regression suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib claude_desktop_logs_winget_command_before_install`

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib codex_desktop_logs_store_command_before_install`

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_cli_logs_npm_command_before_install`

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib opencode_cli_logs_npm_command_before_install`

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib openclaw_logs_official_powershell_command_before_waiting`

Expected: PASS

- [ ] **Step 4: Run manual verification in rollout order**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/test-claude-code-desktop-install.ps1
powershell -ExecutionPolicy Bypass -File scripts/test-codex-desktop-install.ps1
powershell -ExecutionPolicy Bypass -File scripts/test-gemini-cli-install.ps1
powershell -ExecutionPolicy Bypass -File scripts/test-opencode-cli-install.ps1
powershell -ExecutionPolicy Bypass -File scripts/test-openclaw-install.ps1
```

Then verify in the app:

- the active tool card shows progress plus the latest summary message
- the install console shows the exact command
- raw output is expandable
- the latest session remains visible after completion
- detection reflects the local install result

- [ ] **Step 5: Commit**

```bash
git add tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx tests/lib/installSessions.test.ts scripts/test-claude-code-desktop-install.ps1 scripts/test-codex-desktop-install.ps1 scripts/test-gemini-cli-install.ps1 scripts/test-opencode-cli-install.ps1 scripts/test-openclaw-install.ps1
git commit -m "test: verify install console across supported tools"
```
