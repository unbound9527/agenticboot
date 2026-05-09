# Install Console And Tool Verification Design

## Summary

This design adds a reusable install console to AgenticBoot so users can see what the installer is actually doing instead of only seeing a generic progress bar. The console must work for any tool and must not introduce tool-specific UI branches. It also defines a tool-by-tool rollout and validation strategy for the first five targets:

1. Claude Code (Desktop)
2. Codex (Desktop)
3. Gemini CLI
4. OpenCode (CLI)
5. OpenClaw

The scope of this design is installation only. Uninstall changes are explicitly out of scope for this pass.

## Goals

- Show real installation activity in a terminal-like view.
- Keep the current lightweight progress UI, but augment it with structured command and log output.
- Store the most recent install session per tool so users can review success or failure after the task finishes.
- Reuse the same event model, renderer, and verification flow for all tools.
- Add a development-time verification entrypoint for each tool so install logic can be tested in isolation before being relied on by the production UI flow.

## Non-Goals

- No uninstall redesign in this pass.
- No fully persistent install history beyond the most recent session per tool.
- No interactive PTY emulation or full terminal feature set.
- No tool-specific UI components that know how one installer works internally.

## User Experience

### Tool Card

Each tool card keeps the current concise summary:

- phase
- percentage
- latest summary message

When a tool is currently installing, the card also exposes an entrypoint to the install console for that tool. After installation completes or fails, the same entrypoint remains available so the latest session can be reviewed.

### Install Console

The install console is a reusable terminal-style panel. It can live inline on the manager page or inside a dialog if space becomes constrained, but the data contract stays the same.

The console has two views:

- Summary view
  - install phases in time order
  - commands that were executed
  - detected install path
  - final success or failure
  - exit code when available
- Raw view
  - line-oriented event stream
  - command lines
  - stdout lines
  - stderr lines
  - explicit result line

The default presentation favors the summary view, with raw output available through a collapse or tab switch.

### Session Retention

For each tool, the frontend stores the most recent install session:

- while installation is running, the session updates live
- after success or failure, the session remains visible
- starting a new install for the same tool replaces the previous retained session

This is enough to support real-time trust and post-run debugging without building a full history system yet.

## Architecture

### Existing Foundation

The current installer already emits `install-progress` events and the frontend already listens for them. This remains the high-level progress channel.

The project already includes suitable UI primitives:

- `@radix-ui/react-scroll-area`
- `@radix-ui/react-collapsible`
- `@radix-ui/react-accordion`
- existing dialog and card primitives

The first implementation should reuse these pieces instead of introducing a heavy terminal dependency such as `xterm.js`.

### New Event Channel

Add a new backend event stream alongside `install-progress`:

- event name: `install-log`

This event is generic and tool-agnostic. It represents install task activity, not a particular installer implementation.

Suggested shape:

```ts
type InstallLogLevel = "info" | "stdout" | "stderr" | "success" | "error";

type InstallLogKind =
  | "session-started"
  | "phase"
  | "command"
  | "output"
  | "result";

interface InstallLogEvent {
  toolId: string;
  toolName: string;
  sessionId: string;
  timestamp: string;
  phase?: string;
  level: InstallLogLevel;
  kind: InstallLogKind;
  line: string;
  command?: string;
  exitCode?: number | null;
}
```

This protocol is intentionally generic:

- `toolId` and `sessionId` route events
- `phase` links logs to progress
- `kind` distinguishes commands from output and results
- `level` controls presentation
- `command` preserves the actual invoked command line when relevant

### Backend Logging Helpers

The backend should not require every plugin to manually build log strings. Add a common helper layer in the installer service or Windows command helpers that can:

- emit session start
- emit phase transition
- emit command invocation
- emit stdout lines
- emit stderr lines
- emit result with exit code

Plugins then only add installer-specific context when needed, such as:

- "falling back from winget to direct installer download"
- "waiting for official PowerShell script to finish"
- "managed Node.js runtime selected for npm install"

This keeps future tools cheap to integrate.

### Frontend State

Add frontend install-session state keyed by tool id:

```ts
interface InstallLogEntry {
  timestamp: string;
  level: "info" | "stdout" | "stderr" | "success" | "error";
  kind: "session-started" | "phase" | "command" | "output" | "result";
  phase?: string;
  line: string;
  command?: string;
  exitCode?: number | null;
}

interface ToolInstallSession {
  toolId: string;
  toolName: string;
  sessionId: string;
  status: "running" | "complete" | "error";
  startedAt: string;
  endedAt?: string;
  lastSummary?: string;
  installPath?: string;
  entries: InstallLogEntry[];
}
```

Rules:

- only the latest session is retained per tool
- a new `sessionId` replaces the retained session for that tool
- `install-progress` still drives percent and high-level state
- `install-log` builds console content

## Data Flow

1. User starts installation for one tool.
2. Backend emits `install-progress` with a starting phase.
3. Backend emits `install-log` session start event.
4. Plugin or shared command helper emits phase and command events.
5. Command execution emits stdout and stderr lines as output events.
6. Backend emits a result event and final `install-progress`.
7. Frontend marks the session complete or error and keeps the latest session visible for that tool.
8. Detection refresh confirms whether the tool is installed locally.

## Error Handling

- Command launch failures must produce both a user-facing error progress update and a raw error log line.
- If a command exits non-zero, the result event must include the exit code when available.
- If output capture is partial or unavailable, the system still emits phase and command events so users are not left with a frozen-looking install.
- Tools that run opaque installers or remote scripts must emit explanatory phase messages before and after launch so the user understands the waiting state.

## Reuse Strategy

The design must minimize future maintenance cost for new tools:

- one event model for every installer type
- one frontend console renderer for every tool
- one backend logging helper for command execution
- one retention model for latest session per tool
- one validation pattern for isolated install testing

Adding a new tool should usually require:

1. defining install commands and detection logic
2. wiring the plugin into shared logging helpers
3. adding one isolated validation script
4. adding tests

It should not require new UI branches or special frontend state for that tool.

## Development Verification Entry Points

Each target tool should gain a dedicated development verification path that exercises install logic in isolation. The exact implementation can be a PowerShell script, test harness command, or small Rust command wrapper, but all of them should produce the same structured logging behavior as the production flow.

The verification entrypoint for each tool should:

- run only that tool's install path
- print or emit the command actually executed
- surface stdout and stderr
- return a clear success or failure
- be safe to run repeatedly during development

These entrypoints support debugging before using the full UI flow and also make it easier to confirm local installation results one tool at a time.

## Rollout Order

Implement and validate in this order:

1. Claude Code (Desktop)
2. Codex (Desktop)
3. Gemini CLI
4. OpenCode (CLI)
5. OpenClaw

Reasoning:

- desktop installers and store-driven installs are the most opaque, so the install console brings immediate value there
- npm-based CLI tools then benefit from shared command logging helpers
- OpenClaw follows after the generalized logging model is proven, because it already has partial staged progress but still lacks transparent command and output visibility

## Per-Tool Acceptance Criteria

Each tool is considered complete for this pass only when all of the following are true:

- the user can see the actual command or launch action
- the user can see meaningful runtime output or explicit waiting phases
- the latest install session remains reviewable after completion
- the tool can be detected locally after a successful install
- an isolated development verification entrypoint exists
- automated tests cover the new logging and session behavior

## Testing Strategy

### Frontend

- add unit tests for install session state updates from `install-log`
- add component tests for tool card and install console rendering
- verify retained latest-session behavior after completion and failure
- verify summary and raw output views

### Backend

- add tests for shared log emission helpers
- add plugin-specific tests for emitted phase and command events where feasible
- verify opaque installers emit waiting phases before blocking work
- verify result events include error details and exit codes when available

### Manual Validation

For each tool in rollout order:

1. run isolated verification entrypoint
2. confirm actual local installation result
3. run the real UI installation flow
4. confirm live console behavior
5. confirm latest session remains reviewable after completion

## Risks And Mitigations

### Risk: Opaque installers still do not provide rich output

Mitigation:

- emit explicit command and waiting events before blocking calls
- show launch action, installer source, and final exit result even when stdout is limited

### Risk: Log volume becomes noisy in the UI

Mitigation:

- default to summary view
- keep raw view collapsible
- distinguish stdout and stderr visually

### Risk: Future tools accidentally bypass the common model

Mitigation:

- centralize helpers for event emission and command execution
- keep plugin responsibilities narrow
- use acceptance criteria that require the standard console behavior

## Open Questions Resolved By This Design

- Should install progress and raw logs be separate channels
  - yes, keep `install-progress` for summaries and add `install-log` for structured detail
- Should logs be retained after completion
  - yes, retain the latest session per tool
- Should raw output be shown by default
  - no, default to summary and allow raw expansion
- Should isolated verification exist in addition to the real UI flow
  - yes, both are required

## Implementation Boundary

This design only authorizes work for:

- reusable install log protocol
- reusable frontend install console and latest-session retention
- isolated verification entrypoints
- per-tool installation validation for the first five targets

Anything beyond that, especially uninstall redesign or multi-session install history, requires a separate follow-up design.
