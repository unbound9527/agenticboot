# Git

## How AgenticBoot handles Git on Windows

Git is treated as a dependency and is checked before installing tools that rely on it.

- If a working `git` is already available, AgenticBoot reuses it.
- If Git is missing, AgenticBoot installs a managed copy under the selected install root.
- Managed Git installs are kept separate from unrelated system installs.

## What this means for users

- A machine that already has Git does not get a redundant reinstall.
- A clean machine can still complete setup without the user installing Git first.
- Uninstall only targets the AgenticBoot-managed Git directory when applicable.

## Manual verification

```powershell
git --version
```
