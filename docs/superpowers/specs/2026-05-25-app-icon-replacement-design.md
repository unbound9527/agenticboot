# AgenticBoot App Icon Replacement Design

## Goal

Replace the project's current application icon assets with the existing design source at `assets/icons/app-icon-design.svg`, keeping all current icon references and packaging configuration intact.

## Scope

- Use `assets/icons/app-icon-design.svg` as the single source icon artwork.
- Regenerate and replace the Tauri bundle icon assets under `src-tauri/icons/`.
- Replace the in-app About page icon asset at `src/assets/icons/app-icon.png`.
- Preserve existing file paths so no UI or Tauri config changes are required unless verification shows a missing reference.

## Approach

The safest path is to keep the current icon pipeline shape and only swap the generated outputs. This avoids touching `src-tauri/tauri.conf.json`, keeps platform packaging behavior stable, and makes the change easy to review.

## Affected Areas

- `assets/icons/app-icon-design.svg`: source artwork
- `src-tauri/icons/*`: packaged desktop and platform icon assets
- `src/assets/icons/app-icon.png`: app-internal displayed icon
- `src/components/settings/AboutSection.tsx`: reference verification only, no expected code change

## Validation

- Confirm the generated icon files exist at the paths already referenced by Tauri and the frontend.
- Run a TypeScript typecheck to ensure the frontend asset import still resolves.
- Run a targeted app build-side icon generation command and inspect outputs for expected timestamps and file presence.
