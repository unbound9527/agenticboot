# Install Activity Feed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make AgenticBoot's `Wizard` and `Manager` install flows feel continuously active by deriving a recent-activity feed from existing install-log events and surfacing it above the raw console output.

**Architecture:** Keep the current backend event transport unchanged and extend the frontend session reducer so it derives a compact `activity` view from `phase`, `command`, selected `output`, and `result` events. Reuse that derived view in `InstallConsole` and `InstallProgress`, then wire `Wizard` and `Manager` to present the same layered install experience.

**Tech Stack:** React 18, TypeScript, Tauri event listeners, Vitest, Testing Library, shadcn/ui, Lucide

---

## File Structure

- Modify: `src/types/tools.ts`
  - Add a typed `InstallActivityItem` interface to mirror the derived activity layer.
- Modify: `src/hooks/useInstallSessions.ts`
  - Add event promotion, deduplication, and activity derivation on top of retained raw sessions.
- Modify: `src/components/tools/InstallConsole.tsx`
  - Promote the activity feed above the summary/raw tabs and show better active-state messaging.
- Modify: `src/components/tools/InstallProgress.tsx`
  - Highlight the active tool with current-action copy plus recent activity, not just percent and phase.
- Modify: `src/pages/Manager.tsx`
  - Reuse the richer session data when displaying retained consoles for single-tool installs.
- Modify: `src/pages/Wizard.tsx`
  - Ensure the onboarding install view uses the same activity-driven feedback patterns.
- Modify: `tests/lib/installSessions.test.ts`
  - Cover event promotion, deduplication, and terminal-state behavior in the reducer.
- Modify: `tests/components/Manager.installDetection.test.tsx`
  - Cover recent activity visibility during a single-tool install and retained session review.
- Modify: `tests/components/Wizard.installDetection.test.tsx`
  - Cover recent activity visibility during multi-tool onboarding installs.

## Task 1: Add Derived Install Activity To Retained Sessions

**Files:**
- Modify: `src/types/tools.ts`
- Modify: `src/hooks/useInstallSessions.ts`
- Modify: `tests/lib/installSessions.test.ts`

- [ ] **Step 1: Write the failing reducer tests**

Add focused cases to `tests/lib/installSessions.test.ts` that verify:

```ts
it("promotes command and meaningful output into activity items", () => {
  const started = reduceInstallLogEvent(new Map(), {
    toolId: "codex-cli",
    toolName: "Codex (CLI)",
    sessionId: "session-1",
    timestamp: "2026-05-11T09:00:00.000Z",
    level: "info",
    kind: "session-started",
    line: "Install session started",
  });

  const withCommand = reduceInstallLogEvent(started, {
    toolId: "codex-cli",
    toolName: "Codex (CLI)",
    sessionId: "session-1",
    timestamp: "2026-05-11T09:00:01.000Z",
    phase: "installing",
    level: "info",
    kind: "command",
    line: "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
    command: "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
  });

  const withOutput = reduceInstallLogEvent(withCommand, {
    toolId: "codex-cli",
    toolName: "Codex (CLI)",
    sessionId: "session-1",
    timestamp: "2026-05-11T09:00:02.000Z",
    phase: "installing",
    level: "stdout",
    kind: "output",
    line: "Downloading package metadata",
  });

  const session = withOutput.get("codex-cli");
  expect(session?.activity.map((item) => item.line)).toEqual([
    "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
    "Downloading package metadata",
  ]);
});

it("deduplicates repeated low-value output lines", () => {
  const repeated = [
    "Downloading package metadata",
    "Downloading package metadata",
  ];

  let state = new Map<string, ToolInstallSession>();
  for (const [index, line] of repeated.entries()) {
    state = reduceInstallLogEvent(state, {
      toolId: "gemini-cli",
      toolName: "Gemini CLI",
      sessionId: "session-2",
      timestamp: `2026-05-11T09:00:0${index}.000Z`,
      phase: "downloading",
      level: "stdout",
      kind: "output",
      line,
    });
  }

  expect(state.get("gemini-cli")?.activity).toHaveLength(1);
});
```

- [ ] **Step 2: Run the reducer tests to verify they fail**

Run: `node node_modules/vitest/vitest.mjs run tests/lib/installSessions.test.ts`

Expected: FAIL because `ToolInstallSession` does not include `activity` and the reducer does not promote or deduplicate events yet.

- [ ] **Step 3: Add the minimal activity types and reducer logic**

Update `src/types/tools.ts` with:

```ts
export interface InstallActivityItem {
  id: string;
  kind: "phase" | "command" | "signal" | "result";
  phase?: string;
  line: string;
  emphasis: "neutral" | "active" | "success" | "warning" | "error";
  timestamp: string;
}
```

Extend `ToolInstallSession` with:

```ts
activity: InstallActivityItem[];
```

In `src/hooks/useInstallSessions.ts`, add small pure helpers that:

- always promote `command` events
- always promote `result` events
- promote `phase` events when the line changes
- promote `output` events only when the line matches meaningful patterns such as `download`, `install`, `config`, `retry`, `waiting`, `path`, `version`, `complete`, `created`, `failed`, or `error`
- collapse identical consecutive activity lines
- keep the activity list capped to the latest 3 items

- [ ] **Step 4: Run the reducer tests to verify they pass**

Run: `node node_modules/vitest/vitest.mjs run tests/lib/installSessions.test.ts`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/types/tools.ts src/hooks/useInstallSessions.ts tests/lib/installSessions.test.ts
git commit -m "feat: derive install activity feed"
```

## Task 2: Redesign InstallConsole Around The Activity Feed

**Files:**
- Modify: `src/components/tools/InstallConsole.tsx`
- Modify: `tests/components/Manager.installDetection.test.tsx`

- [ ] **Step 1: Write the failing component test for visible recent activity**

Add a test to `tests/components/Manager.installDetection.test.tsx` like:

```tsx
it("shows recent activity above the raw console output", async () => {
  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <Manager />
    </QueryClientProvider>,
  );

  await screen.findByText("Codex (CLI)");

  act(() => {
    installLogListener?.({
      toolId: "codex-cli",
      toolName: "Codex (CLI)",
      sessionId: "session-1",
      timestamp: "2026-05-11T09:00:00.000Z",
      phase: "installing",
      level: "info",
      kind: "command",
      line: "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
      command: "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
    });
  });

  expect(await screen.findByText(/Recent activity/i)).toBeInTheDocument();
  expect(await screen.findByText(/npm install -g @openai\/codex/i)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the manager test to verify it fails**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx`

Expected: FAIL because `InstallConsole` does not render a recent-activity section yet.

- [ ] **Step 3: Implement the minimal console redesign**

In `src/components/tools/InstallConsole.tsx`:

- add a `Recent activity` block above the tabs
- render `session.activity` with a clear running/success/error emphasis
- keep the existing `Summary` and `Raw Output` tabs intact
- preserve auto-expand while running
- keep the empty state lightweight until the first promoted activity arrives

Prefer a simple structure:

```tsx
{session.activity.length > 0 ? (
  <div className="border-b border-border/60 px-4 py-3">
    <p className="text-[11px] font-medium text-foreground">Recent activity</p>
    <div className="mt-2 space-y-1.5">
      {session.activity.map((item) => (
        <div key={item.id} className="flex gap-2 text-[11px]">
          <span className="text-muted-foreground">{item.phase ? `[${item.phase}]` : "[log]"}</span>
          <span>{item.line}</span>
        </div>
      ))}
    </div>
  </div>
) : null}
```

- [ ] **Step 4: Run the manager test to verify it passes**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/tools/InstallConsole.tsx tests/components/Manager.installDetection.test.tsx
git commit -m "feat: show install activity feed in console"
```

## Task 3: Add Current-Action And Recent-Activity Feedback To InstallProgress

**Files:**
- Modify: `src/components/tools/InstallProgress.tsx`
- Modify: `tests/components/Wizard.installDetection.test.tsx`

- [ ] **Step 1: Write the failing wizard test for active-install feedback**

Add a test to `tests/components/Wizard.installDetection.test.tsx` like:

```tsx
it("shows current action and recent activity while a tool is still installing", async () => {
  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <Wizard onComplete={vi.fn()} initialSelectedToolIds={["codex-cli"]} />
    </QueryClientProvider>,
  );

  await waitFor(() => {
    expect(toolsApiMock.detectTools).toHaveBeenCalled();
  });

  fireEvent.click(await screen.findByRole("button", { name: /install/i }));

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
      timestamp: "2026-05-11T09:00:00.000Z",
      phase: "installing",
      level: "info",
      kind: "command",
      line: "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
      command: "npm install -g @openai/codex --prefix D:\\AgenticTools\\codex-cli",
    });
  });

  expect(await screen.findByText(/Running npm install/i)).toBeInTheDocument();
  expect(await screen.findByText(/npm install -g @openai\/codex/i)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the wizard test to verify it fails**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Wizard.installDetection.test.tsx`

Expected: FAIL because `InstallProgress` does not surface a dedicated current-action block with recent activity.

- [ ] **Step 3: Implement the minimal active-install focus area**

In `src/components/tools/InstallProgress.tsx`:

- derive the active tool session and progress together
- render a current-action section above the per-tool list
- use `progress.message` as the first choice for the main sentence
- fall back to the newest promoted activity line when the progress message is generic
- show up to 3 recent activity lines for the active session
- keep overall progress and per-tool statuses unchanged

- [ ] **Step 4: Run the wizard test to verify it passes**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Wizard.installDetection.test.tsx`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/tools/InstallProgress.tsx tests/components/Wizard.installDetection.test.tsx
git commit -m "feat: highlight active install activity"
```

## Task 4: Align Wizard And Manager Wiring With The Shared Activity Model

**Files:**
- Modify: `src/pages/Manager.tsx`
- Modify: `src/pages/Wizard.tsx`
- Modify: `tests/components/Manager.installDetection.test.tsx`
- Modify: `tests/components/Wizard.installDetection.test.tsx`

- [ ] **Step 1: Write the failing integration assertions for retained and live activity**

Add assertions that prove:

- `Manager` reopens the latest retained console for an installed tool without losing the recent activity section
- `Wizard` still selects and starts installs normally while the richer activity UI is active

Use the existing mocked listeners and extend one manager test plus one wizard test instead of introducing new broad suites.

- [ ] **Step 2: Run both component test files to verify they fail on the new assertions**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx`

Expected: FAIL because page wiring does not consistently prefer the richer shared session presentation yet.

- [ ] **Step 3: Make the minimal page-level wiring changes**

In `src/pages/Manager.tsx`:

- keep using `useInstallSessions`
- prefer the retained session's `activity` for open-console presentation
- keep pending-install and refresh behavior unchanged

In `src/pages/Wizard.tsx`:

- pass the active retained session into `InstallProgress` as it does today
- avoid any page-local formatting that duplicates the reducer's activity logic

- [ ] **Step 4: Run both component test files to verify they pass**

Run: `node node_modules/vitest/vitest.mjs run tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/pages/Manager.tsx src/pages/Wizard.tsx tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx
git commit -m "feat: unify install activity across wizard and manager"
```

## Task 5: Run Final Verification On The Install UX Slice

**Files:**
- Modify: `docs/superpowers/plans/2026-05-11-install-activity-feed.md`

- [ ] **Step 1: Run the targeted install-session and component tests**

Run: `node node_modules/vitest/vitest.mjs run tests/lib/installSessions.test.ts tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx`

Expected: PASS with all targeted tests green.

- [ ] **Step 2: Run a broader confidence check for the install-related unit suite**

Run: `pnpm test:unit -- --runInBand tests/lib/installSessions.test.ts tests/components/Manager.installDetection.test.tsx tests/components/Wizard.installDetection.test.tsx`

Expected: PASS, or if the project command shape differs, the nearest equivalent single-run unit test command should pass with exit code 0.

- [ ] **Step 3: Mark the plan complete**

Update this file so each completed checkbox is checked before final handoff.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/plans/2026-05-11-install-activity-feed.md
git commit -m "docs: mark install activity plan complete"
```
