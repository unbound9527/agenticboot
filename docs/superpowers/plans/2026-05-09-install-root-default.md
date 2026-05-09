# Install Root Default Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change the install wizard's default root from `D:\AITools` to `D:\AgenticBoot` for sessions without a saved install root, while preserving saved-root override behavior.

**Architecture:** Keep the change entirely in the frontend. Update the active wizard entry point in `src/pages/Wizard.tsx`, align the legacy `PathConfig` component in `src/components/tools/PathConfig.tsx`, and prove the behavior with focused Vitest component tests so the default cannot silently drift back.

**Tech Stack:** React 18, TypeScript, Vitest, Testing Library, Tauri frontend APIs

---

## File Structure

- `src/pages/Wizard.tsx`
  Active install wizard screen. Owns the initial `rootPath` state, loads any saved install root from `toolsApi.getInstallRoot()`, and kicks off tool detection using the current root.
- `tests/components/Wizard.installDetection.test.tsx`
  Existing regression coverage for wizard install-root behavior and detection refresh behavior. This is the right place to lock the new default root behavior.
- `src/components/tools/PathConfig.tsx`
  Secondary path-picking component that still hardcodes the old root in both its default value and placeholder.
- `tests/components/PathConfig.test.tsx`
  New focused component test file to pin the `PathConfig` default input value, placeholder, and preview text to `D:\AgenticBoot`.

### Task 1: Update Wizard Default Root Through TDD

**Files:**
- Modify: `tests/components/Wizard.installDetection.test.tsx`
- Modify: `src/pages/Wizard.tsx`
- Test: `tests/components/Wizard.installDetection.test.tsx`

- [ ] **Step 1: Write the failing wizard regression test and update root-specific expectations**

```tsx
it("uses D:\\AgenticBoot as the default install root when no saved root exists", async () => {
  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <Wizard onComplete={vi.fn()} />
    </QueryClientProvider>,
  );

  expect(
    await screen.findByDisplayValue("D:\\AgenticBoot"),
  ).toBeInTheDocument();

  await waitFor(() => {
    expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
      [...TOOL_IDS],
      "D:\\AgenticBoot",
    );
  });
});
```

```tsx
// In the existing wizard detection tests, replace the old visible default root:
fireEvent.change(screen.getByDisplayValue("D:\\AgenticBoot"), {
  target: { value: "E:\\CustomTools" },
});
```

- [ ] **Step 2: Run the wizard test file to verify it fails for the expected reason**

Run:

```bash
pnpm exec vitest run tests/components/Wizard.installDetection.test.tsx
```

Expected:
- FAIL because the rendered input still shows `D:\AITools`
- At least one assertion or mock-call expectation still reports `D:\AITools` instead of `D:\AgenticBoot`

- [ ] **Step 3: Write the minimal wizard implementation**

```tsx
const DEFAULT_ROOT = "D:\\AgenticBoot";
```

```tsx
<Input
  id="install-root"
  value={rootPath}
  onChange={(e) => setRootPath(e.target.value)}
  placeholder="D:\\AgenticBoot"
  className="font-mono text-sm"
/>
```

- [ ] **Step 4: Run the wizard test file to verify it passes**

Run:

```bash
pnpm exec vitest run tests/components/Wizard.installDetection.test.tsx
```

Expected:
- PASS
- The new default-root regression test passes
- The existing persisted-root test still passes, proving saved values still override the default

- [ ] **Step 5: Commit the wizard change**

```bash
git add tests/components/Wizard.installDetection.test.tsx src/pages/Wizard.tsx
git commit -m "test: lock wizard install root default"
```

### Task 2: Align PathConfig Default Root Through TDD

**Files:**
- Create: `tests/components/PathConfig.test.tsx`
- Modify: `src/components/tools/PathConfig.tsx`
- Test: `tests/components/PathConfig.test.tsx`

- [ ] **Step 1: Write the failing `PathConfig` regression test**

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PathConfig } from "@/components/tools/PathConfig";

describe("PathConfig", () => {
  it("uses D:\\AgenticBoot as the default root in the input and preview", () => {
    render(<PathConfig onNext={vi.fn()} onBack={vi.fn()} />);

    const input = screen.getByDisplayValue("D:\\AgenticBoot");
    expect(input).toHaveAttribute("placeholder", "D:\\AgenticBoot");

    const preview = screen.getByText(/claude-code-cli/).closest("pre");
    expect(preview).toHaveTextContent("D:\\AgenticBoot");
  });
});
```

- [ ] **Step 2: Run the `PathConfig` test to verify it fails for the expected reason**

Run:

```bash
pnpm exec vitest run tests/components/PathConfig.test.tsx
```

Expected:
- FAIL because the component still initializes with `D:\AITools`
- The placeholder and preview assertions also reflect the old root

- [ ] **Step 3: Write the minimal `PathConfig` implementation**

```tsx
const DEFAULT_ROOT = 'D:\\AgenticBoot';
```

```tsx
<Input
  id="install-root"
  value={rootPath}
  onChange={(e) => setRootPath(e.target.value)}
  placeholder="D:\\AgenticBoot"
  className="font-mono text-sm"
/>
```

- [ ] **Step 4: Run the `PathConfig` test to verify it passes**

Run:

```bash
pnpm exec vitest run tests/components/PathConfig.test.tsx
```

Expected:
- PASS
- The input value, placeholder, and preview all reflect `D:\AgenticBoot`

- [ ] **Step 5: Commit the `PathConfig` change**

```bash
git add tests/components/PathConfig.test.tsx src/components/tools/PathConfig.tsx
git commit -m "test: align path config install root default"
```

### Task 3: Final Verification

**Files:**
- Test: `tests/components/Wizard.installDetection.test.tsx`
- Test: `tests/components/PathConfig.test.tsx`

- [ ] **Step 1: Run both focused component test files together**

Run:

```bash
pnpm exec vitest run tests/components/Wizard.installDetection.test.tsx tests/components/PathConfig.test.tsx
```

Expected:
- PASS
- No failures in the wizard or `PathConfig` regression coverage

- [ ] **Step 2: Run frontend typecheck**

Run:

```bash
pnpm typecheck
```

Expected:
- PASS
- No TypeScript errors introduced by the new test file or updated component literals

- [ ] **Step 3: Recheck the spec acceptance criteria against the finished diff**

Checklist:

```text
[ ] New wizard session with no saved install root shows D:\AgenticBoot
[ ] Saved install root still overrides the default
[ ] No persisted install root values were migrated or rewritten
```
