# Install Activity Feed Design

> Scope: improve the installation experience in AgenticBoot's `Wizard` and `Manager` so the page feels continuously active during installs, with richer user-facing activity summaries and a clearer bridge from high-level progress to raw logs.

## Goal

Make the install flow feel smooth and trustworthy even when installers are slow, chatty, or temporarily silent by:

- surfacing a compact real-time activity feed for the active tool
- preserving raw logs for debugging without making them the default experience
- reusing the same interaction model in both `Wizard` and `Manager`
- staying inside AgenticBoot's current install-progress and install-log architecture

## Relationship To Existing Install Console Work

The repository already introduced an install console and retained latest-session model in the 2026-05-09 install-console design. This design is a follow-up focused on interaction quality rather than event transport.

The earlier design answered:

- how to emit and retain install logs
- how to render summary and raw output
- how to reuse the console across tools

This design answers:

- how to keep the page feeling alive between large progress jumps
- how to promote the most useful events into a user-readable activity stream
- how to keep `Wizard` and `Manager` visually and behaviorally aligned during installs

## Current Problems

The current install experience already has the right primitives:

- `install-progress` provides phase, percent, and short messages
- `install-log` provides session, phase, command, output, and result events
- `InstallConsole` already renders summary and raw output
- `Wizard` and `Manager` already subscribe to install progress and retained sessions

The friction is in presentation rather than missing backend events:

1. The page often looks stalled when percent does not move for a while.
2. Important proof-of-life signals such as command execution and useful output lines are hidden in the raw log tab.
3. `Wizard` and `Manager` expose similar install state with different levels of emphasis, so the experience does not feel unified.
4. Summary text is too sparse to reassure users during slow installers such as npm, PowerShell scripts, and desktop installers.

## User Experience Principles

### 1. Always Show Active Work

If the backend is still producing command or meaningful output events, the page should look active even when numeric progress is temporarily unchanged.

### 2. Default To Human-Readable Signals

Users should first see "what the installer is doing now" and "what just happened". Raw logs remain available, but they should not be the only source of reassurance.

### 3. Preserve Debuggability

The design must not throw away details. It should layer the experience:

- top layer: current action and recent activity
- middle layer: concise session timeline
- bottom layer: full raw output

### 4. Reuse Shared Logic

`Wizard` and `Manager` should use the same event-to-activity transformation and the same console component behavior. This is an AgenticBoot install-flow concern, not a page-specific special case.

## Proposed Interaction Model

Each active install surface should show three coordinated levels of feedback.

### A. Current Action Banner

For the active tool, show a stronger live status block above or within the progress area:

- tool name
- current phase label
- progress percent when available
- primary user-facing sentence, derived from progress message or latest meaningful activity

Examples:

- `Codex (CLI) is running npm install`
- `OpenClaw is executing the official install script`
- `Gemini CLI is finishing managed files`

This banner remains visible while a tool is active and settles into a success, skipped, or error state immediately when the terminal result arrives.

### B. Recent Activity Feed

Add a short rolling activity list for the active session that shows the latest high-value events. This becomes the default "alive" indicator.

Rules:

- retain only the most recent few display entries, not the whole raw stream
- include command events by default
- include selected output lines when they contain user-relevant signals
- collapse repeated low-value lines
- timestamp entries relative to session order rather than requiring users to read exact times

Example feed:

- `Running: npm install -g @openai/codex --prefix D:\AgenticTools\codex-cli`
- `npm reported package download progress`
- `Managed shim created`

### C. Expandable Full Console

Keep the existing console concept, but reposition it as the detailed layer below the activity feed:

- default open while a tool is running
- summary tab updated to include richer timeline entries
- raw tab unchanged in spirit, still showing all retained events

This lets the top of the install UI stay readable while preserving the underlying debug story.

## Event Promotion Rules

The backend does not need a new event channel for this iteration. The frontend should reinterpret existing events into an activity model.

### Always Promote

- `command`
- `result`
- `phase` when the phase label changes meaningfully

### Conditionally Promote `output`

Promote an output event into the visible activity feed only when it matches at least one of these categories:

- download/install/configure progress
- waiting or retry notices
- path/location information
- detected version or executable details
- completion markers
- actionable warnings or failures

Everything else remains in raw output only.

### Deduplication

To avoid noisy feeds:

- repeated identical lines should collapse
- near-identical progress chatter should update the latest display item rather than append indefinitely
- terminal success/failure should replace transient "still running" emphasis

## Shared Frontend Data Model

The existing retained session state should grow a lightweight derived view for display.

Suggested additions:

```ts
interface InstallActivityItem {
  id: string;
  kind: "phase" | "command" | "signal" | "result";
  phase?: string;
  line: string;
  emphasis: "neutral" | "active" | "success" | "warning" | "error";
  timestamp: string;
}

interface ToolInstallSession {
  toolId: string;
  toolName: string;
  sessionId: string;
  status: "running" | "complete" | "error";
  startedAt: string;
  endedAt?: string;
  lastSummary?: string;
  entries: InstallLogEvent[];
  activity: InstallActivityItem[];
}
```

This derived `activity` array should be built in the session reducer layer so both pages render from the same interpretation.

## Component Responsibilities

### `useInstallSessions`

- retain the latest raw session per tool
- derive display activity items from install-log events
- apply deduplication and low-noise filtering
- keep terminal result state authoritative

### `InstallConsole`

- show the recent activity feed prominently near the top
- keep `Summary` and `Raw Output` tabs below
- auto-expand while active
- show a strong empty/loading state only before the first meaningful event arrives

### `InstallProgress`

- treat the active tool as a first-class focus area
- show current-action copy plus recent activity
- continue to compute overall progress across all plan steps
- avoid relying on percentage movement alone as proof of activity

### `Wizard`

- use the shared activity model during multi-tool onboarding installs
- keep the install page visibly active when the current tool is still emitting command or output signals

### `Manager`

- use the same activity model for one-off installs and retained session review
- let users reopen the latest console without losing the higher-level summary cues

## Visual And Behavioral Guidance

The goal is not a heavy terminal simulation. The install flow should feel more like a guided operations panel than a full shell.

Recommended behavior:

- keep the active tool visually highlighted
- use concise icons and subtle motion for running state
- show at most 3 recent activity items by default
- separate current action, recent activity, and raw logs with clear visual hierarchy
- avoid flashing or constantly reordering the whole panel

## Error Handling

The new activity model must make errors easier to understand, not easier to miss.

Rules:

- error result events always appear in the visible activity feed
- stderr-like output promoted into activity should render with warning/error emphasis
- if raw logs exist but no activity-qualified output exists yet, the UI should still show the latest command or phase so the page does not appear frozen

## Testing Strategy

Testing should stay on the AgenticBoot install path:

- reducer tests for event promotion, deduplication, and terminal-state behavior
- component tests for `InstallConsole` activity rendering
- `Wizard` and `Manager` tests proving that command/output/result events keep visible UI feedback moving

The design does not require new end-to-end infrastructure or backend event schema changes unless implementation reveals a true gap.

## Acceptance Criteria

This work is complete when all of the following are true:

- active installs in both `Wizard` and `Manager` show a visible recent-activity layer in addition to the progress bar
- command events are visible without requiring users to open raw output
- selected meaningful output lines appear in the human-readable activity feed
- repeated noisy output does not flood the default view
- raw output remains available for debugging
- automated tests cover event promotion, deduplication, and visible activity updates
