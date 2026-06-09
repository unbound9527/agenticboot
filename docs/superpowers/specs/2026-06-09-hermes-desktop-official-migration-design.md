# Hermes Desktop Official Migration Design

> Scope: replace AgenticBoot's current Hermes integration with the official Nous Research `Hermes Desktop` product across installation, detection, update, uninstall, launch, and provider-configuration flows. This migration intentionally drops support for the old third-party `fathah/hermes-desktop` route and any "Hermes (Web UI)" product framing.

## Goal

Make AgenticBoot treat Hermes as a first-class official desktop app instead of a legacy web UI wrapper or third-party desktop shell.

This migration must ensure that:

- new installs use the official Hermes Desktop distribution
- update checks and update execution use the official release source
- detection only recognizes official Hermes Desktop installs
- uninstall only targets official Hermes Desktop installs
- launch actions open official Hermes Desktop
- provider, memory, and config-management messaging consistently points users to Hermes Desktop

## Non-Goals

This work does not attempt to:

- preserve compatibility with `fathah/hermes-desktop`
- auto-migrate third-party installs into official installs
- add a second Hermes tool id or dual-catalog entry
- redesign Hermes provider UX beyond terminology, launch routing, and consistency fixes

## Product Decision

AgenticBoot keeps the internal tool id `hermes` to minimize churn in the existing install pipeline, state cache, provider bindings, and UI routing.

However, the product represented by that id becomes only:

- display name: `Hermes Desktop`
- install source: official Nous release channel
- supported install type: official Hermes Desktop only

Anything that only matches the legacy third-party desktop shell should be treated as unsupported and effectively not installed.

## Current Problems

The repository already partially shifted toward Hermes Desktop, but the implementation is mixed:

- the plugin metadata and launcher mostly say `Hermes Desktop`
- some tests and UI strings still say `Hermes (Web UI)`
- the installer/update source still points at `fathah/hermes-desktop`
- detection and uninstall logic still tolerates old executable names and old install locations

This creates three concrete risks:

1. AgenticBoot may install or update from the wrong upstream.
2. The UI presents Hermes inconsistently across catalog, detection, and configuration.
3. Old third-party installs may be mistaken for supported official installs.

## User-Facing Behavior After Migration

### Tool Catalog

The Hermes card should consistently present:

- name: `Hermes Desktop`
- description: official Hermes desktop app from Nous Research
- install strategy: desktop installer

There should be no remaining catalog or test copy that labels the tool as `Hermes (Web UI)`.

### Install

On Windows, install Hermes via the official Hermes Desktop installer published on the Hermes download site.

Expected behavior:

- download the current Windows installer from the official Hermes asset host
- run the installer silently where supported
- record the install as AgenticBoot-managed when installed under the selected install root

If the official installer URL is stable and versionless, install execution should prefer the stable direct asset URL over scraping third-party GitHub releases.

### Detection

Detection should only return installed for official Hermes Desktop signals.

Recognized signals should include:

- official install directories
- official uninstall registry entries
- official executable names

Legacy third-party install names, directories, or release-specific assumptions should not count as installed.

### Update

Update checks should use the official release source rather than `fathah/hermes-desktop`.

Because the official Windows download is distributed from the Hermes site and asset host rather than a simple GitHub release API already used elsewhere in AgenticBoot, Hermes needs its own update-source strategy.

Expected outcomes:

- `check_tool_updates` can resolve the latest official Hermes Desktop version
- Hermes update execution downloads the official installer and re-runs install/update through the same official path
- version comparison remains normalized the same way as other tools

### Uninstall

Uninstall should only target official Hermes Desktop uninstall entries or local uninstaller binaries associated with official installs.

If no official uninstall signal is found, AgenticBoot should fail with a clear message rather than attempting best-effort removal of unsupported legacy installs.

### Launch

The existing "open Hermes Desktop" flow remains, but all fallback path probing should prefer official install paths and official executable names only.

### Provider And Config Messaging

Hermes provider and memory flows already depend on the official Hermes config layout and should remain in place.

This migration should make the messaging fully consistent:

- edit/remove provider hints should always reference `Hermes Desktop`
- config-opening buttons should reference `Hermes Desktop`
- any helper text implying a web UI should be removed

## Technical Design

## 1. Plugin Metadata And Catalog Identity

Primary files:

- `src-tauri/src/plugins/hermes.rs`
- `src-tauri/src/tool_types.rs`
- `src-tauri/src/plugin.rs`
- frontend tests that build fake tool catalogs

Required changes:

- keep plugin id as `hermes`
- keep plugin metadata name as `Hermes Desktop`
- update description text to clearly indicate the official desktop app
- update all frontend catalog fixtures and tests to use `Hermes Desktop`

## 2. Official Install Source

Primary file:

- `src-tauri/src/plugins/hermes.rs`

Required changes:

- remove `fetch_latest_hermes_version()` logic that queries `fathah/hermes-desktop`
- remove GitHub asset-name construction based on third-party release naming
- switch Windows install download to the official Hermes asset URL

Current verified official Windows artifact:

- `https://hermes-assets.nousresearch.com/Hermes-Setup.exe`

Design choice:

- use the stable official asset URL for install execution on Windows
- do not depend on a third-party GitHub release tag to construct the download URL

If the code still needs a version for update checks, obtain it from an official source separately rather than inferring it from the download URL.

## 3. Official Update Source Model

Primary files:

- `src-tauri/src/plugins/hermes.rs`
- `src-tauri/src/services/installer/mod.rs`
- `src\types\tools.ts`

Required changes:

- add support for a Hermes-specific official update source kind, for example `hermes-official`
- teach update checking to fetch the latest Hermes version from an official Hermes page or metadata source
- keep GitHub and npm update strategies unchanged for other tools

Recommended source for latest version:

- official download/desktop page or another official Hermes endpoint that exposes the current version

Fallback rule:

- if the latest official version cannot be resolved, skip update reporting rather than using the legacy GitHub repo

## 4. Detection Tightening

Primary files:

- `src-tauri/src/plugins/hermes.rs`
- `src-tauri/src/commands/hermes.rs`

Required changes:

- remove detection paths that only exist for legacy third-party installs
- narrow candidate directories to official install locations
- narrow executable-name matching to official Hermes Desktop binary names
- narrow registry matching to official display names and official install roots

Behavioral rule:

- unsupported old third-party installs must not surface as installed tools

## 5. Uninstall Tightening

Primary file:

- `src-tauri/src/plugins/hermes.rs`

Required changes:

- only use official uninstall registry entries or official uninstaller executables
- remove permissive fallback behavior that could silently treat unsupported legacy directories as uninstallable Hermes installs
- keep managed-root cleanup behavior only when the install is clearly an official managed install

## 6. Launch Consistency

Primary files:

- `src-tauri/src/commands/hermes.rs`
- `src/hooks/useHermes.ts`

Required changes:

- align launcher probing with the tightened official detection rules
- keep the current localized "not found" and "open failed" flows
- ensure the command no longer treats legacy third-party locations as valid

## 7. Provider And Memory Integration

Primary files:

- `src-tauri/src/hermes_config.rs`
- `src/components/providers/`
- `src/components/hermes/`
- `src/lib/api/hermes.ts`

Most of this area already assumes official Hermes config ownership, so the migration here is mainly consistency work:

- confirm no remaining strings refer to `Hermes (Web UI)`
- confirm provider-management hints consistently point to Hermes Desktop
- confirm opening config and memory panels still launch Hermes Desktop

No schema migration is required for Hermes provider data in AgenticBoot because the internal app id remains `hermes`.

## 8. Detection Cache And Installed State

Primary files:

- `src-tauri/src/commands/tools.rs`
- install-detection tests

Because the tool id remains `hermes`, existing cached records may still exist in local databases.

Desired behavior:

- future detections should overwrite cached state based on official-only detection results
- unsupported legacy installs should naturally age out to `not_installed`
- no dedicated DB migration is required as long as detection truth becomes authoritative

## Testing Plan

Priority tests to update or add:

- `tests/components/Manager.installDetection.test.tsx`
- `tests/components/Wizard.installDetection.test.tsx`
- Hermes plugin unit tests in `src-tauri/src/plugins/hermes.rs`
- any launcher tests in `src-tauri/src/commands/hermes.rs`
- update-check tests covering the new official Hermes update-source kind

Test expectations:

- catalog name is `Hermes Desktop`
- official install paths are detected
- legacy third-party install paths are not detected
- uninstall works for official installs
- update source does not point to `fathah/hermes-desktop`
- provider/config UI copy consistently references Hermes Desktop

## Risks

### 1. Official Version Discovery May Be Less Structured

Unlike GitHub releases, the official desktop download path may not expose a simple versioned JSON API already used by AgenticBoot.

Mitigation:

- separate install URL resolution from version resolution
- allow update checks to fail closed for Hermes without breaking install

### 2. Silent Install Behavior May Differ

The official installer may not behave exactly like the legacy installer regarding silent flags or install directory overrides.

Mitigation:

- verify supported silent flags before claiming managed-root install parity
- if the installer ignores custom target paths, reflect that clearly in managed/external install handling

### 3. Existing Legacy Users Lose Auto-Recognition

This is intentional per product decision, but it changes observed behavior.

Mitigation:

- make the unsupported state explicit in error copy where helpful
- keep the UI truthful rather than pretending old installs are supported

## Open Implementation Questions

These must be resolved during implementation verification, not deferred:

1. Does the official Windows installer support the current silent flags and custom `/D=<path>` target directory?
2. Which official endpoint is the most stable source for the current Hermes Desktop version?
3. What exact Windows executable and uninstall registry names does the official installer register on a real machine?

## Recommended Implementation Order

1. Replace Hermes catalog/test/UI copy with `Hermes Desktop`.
2. Switch install execution from legacy GitHub releases to the official asset host.
3. Introduce Hermes-specific official update-source handling.
4. Tighten detection, launch, and uninstall logic to official signals only.
5. Update tests to enforce the new behavior.
6. Run typecheck and relevant unit tests for install detection and Hermes flows.

## Success Criteria

The migration is complete when:

- AgenticBoot no longer references `fathah/hermes-desktop` anywhere in runtime logic
- AgenticBoot no longer labels the tool as `Hermes (Web UI)`
- Hermes install, detect, update, uninstall, and launch flows all target the official Hermes Desktop product
- provider/config UI copy is internally consistent with Hermes Desktop
- tests explicitly cover the official-only support boundary
