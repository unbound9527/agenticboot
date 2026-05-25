# App Icon Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current app icon assets with outputs generated from `assets/icons/app-icon-design.svg` without changing existing asset references.

**Architecture:** Keep the current frontend and Tauri icon paths unchanged, and regenerate the files those paths already point to. Use the existing Tauri icon toolchain where possible so packaging behavior remains aligned with the project's current desktop build setup.

**Tech Stack:** Tauri CLI, Vite asset pipeline, PowerShell, TypeScript

---

### Task 1: Confirm current icon entrypoints

**Files:**
- Modify: `src-tauri/tauri.conf.json` (verification only, no expected content change)
- Modify: `src/components/settings/AboutSection.tsx` (verification only, no expected content change)

- [ ] **Step 1: Verify bundle icon paths**

Run: `Get-Content src-tauri\tauri.conf.json`
Expected: `bundle.icon` points at `src-tauri/icons/32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, and `icon.ico`.

- [ ] **Step 2: Verify frontend icon path**

Run: `Get-Content src\components\settings\AboutSection.tsx`
Expected: the component imports `@/assets/icons/app-icon.png`.

### Task 2: Regenerate icon assets from the SVG source

**Files:**
- Modify: `src-tauri/icons/*`
- Modify: `src/assets/icons/app-icon.png`

- [ ] **Step 1: Run icon generation from the SVG source**

Run: `npx tauri icon assets/icons/app-icon-design.svg src-tauri/icons`
Expected: Tauri icon assets are regenerated in `src-tauri/icons/`.

- [ ] **Step 2: Derive the frontend PNG from the generated icon set**

Run: `Copy-Item src-tauri\icons\128x128.png src\assets\icons\app-icon.png -Force`
Expected: `src/assets/icons/app-icon.png` matches the refreshed icon artwork.

### Task 3: Verify generated assets and frontend integration

**Files:**
- Modify: `src/assets/icons/app-icon.png`
- Modify: `src-tauri/icons/*`

- [ ] **Step 1: Check file presence and sizes**

Run: `Get-ChildItem src-tauri\icons | Select-Object Name,Length,LastWriteTime`
Expected: icon outputs exist and show fresh write times.

- [ ] **Step 2: Run typecheck**

Run: `pnpm typecheck`
Expected: exit code 0.
