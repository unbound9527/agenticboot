# CC Switch Dependency Compatibility Design

**Goal:** Keep AgenticBoot as close as practical to the upstream `CC Switch` dependency surface while preserving the current startup baseline and minimizing future merge conflict risk.

## Problem Statement

The current repository has already diverged from upstream in a small set of Tauri-related dependency declarations. Those changes reduce merge compatibility and may contribute to uncertainty when the app starts. The goal is not to blindly pin every package to upstream, but to keep the dependency delta as small and intentional as possible.

## Constraints

- Preserve the current startup baseline. Any dependency change must keep `pnpm install --frozen-lockfile` and desktop startup working.
- Keep the merge surface with upstream `CC Switch` as small as possible.
- Limit changes to dependency declarations and generated lock data unless a runtime failure proves a wider fix is required.
- Do not change unrelated UI, install-flow, or data-layer code as part of this compatibility pass.

## Decision

Use the current working versions as the baseline, then narrow the dependency declarations toward upstream-compatible ranges only where the runtime behavior remains stable.

This means:

- Keep the actual resolved versions that are already known to work unless there is a clear compatibility reason to move.
- Prefer upstream-style semver ranges for dependencies that are shared with `CC Switch`.
- Avoid broad dependency upgrades outside the Tauri family during this pass.

## Scope

### In scope

- `package.json`
- `pnpm-lock.yaml`
- `src-tauri/Cargo.toml`
- `Cargo.lock` only if Cargo dependency resolution changes
- Startup verification through the existing dev script

### Out of scope

- UI redesign
- Feature work
- Windows installer behavior changes
- Database schema changes
- Non-Tauri dependency refreshes

## Success Criteria

- The dependency declarations remain compatible with upstream `CC Switch` where possible.
- The app still starts through the repository-managed dev script.
- The lockfiles remain internally consistent.
- Any remaining startup failure is clearly attributable to runtime logic rather than dependency drift.

