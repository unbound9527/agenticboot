# Install Root Default Design

**Date:** 2026-05-09

## Goal

Change the default install root shown in the Windows install wizard from `D:\AITools` to `D:\AgenticBoot` for new sessions that do not already have a saved install root.

## Scope

- Update the install wizard's frontend default root value.
- Keep saved install root values authoritative when they already exist.
- Align any inactive or secondary frontend path input component that still hardcodes the old default so the codebase does not contain conflicting defaults.

## Out Of Scope

- Migrating previously saved `install_root` values.
- Changing database records or other persisted settings.
- Changing uninstall behavior, managed-path ownership rules, or backend install logic.

## Current State

- The backend fallback in `src-tauri/src/database/dao/tools.rs` already returns `D:\AgenticBoot` when no install root has been saved.
- The active frontend wizard in `src/pages/Wizard.tsx` still initializes its local default to `D:\AITools`.
- `src/components/tools/PathConfig.tsx` also still hardcodes `D:\AITools` for both its default value and placeholder text.

## Proposed Change

### Approach

Use `D:\AgenticBoot` as the frontend default install root everywhere the wizard UI currently hardcodes `D:\AITools`.

### Behavior

- When no saved install root exists:
  - The wizard should render `D:\AgenticBoot` as the initial input value.
- When a saved install root exists:
  - The saved value should continue to replace the frontend default after loading.
- Placeholder text and any preview text should match the new default root.

## Implementation Notes

- Update `DEFAULT_ROOT` in `src/pages/Wizard.tsx` from `D:\AITools` to `D:\AgenticBoot`.
- Update the matching default and placeholder in `src/components/tools/PathConfig.tsx`.
- Do not modify backend persistence or add migration logic because the requested behavior only applies to new wizard sessions without saved configuration.

## Testing

- Add or update a frontend test that verifies the wizard shows `D:\AgenticBoot` when `toolsApi.getInstallRoot()` returns no saved value.
- Keep existing behavior coverage for saved install roots if already present.

## Risks

- Low risk. The main failure mode is changing only one frontend entry point and leaving another hardcoded to the old root, which would reintroduce inconsistent defaults later.

## Acceptance Criteria

- A new wizard session with no saved install root shows `D:\AgenticBoot`.
- A saved install root, if present, still overrides the default.
- No persisted install root values are migrated or rewritten.
