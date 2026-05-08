# OpenClaw

## How AgenticBoot handles OpenClaw on Windows

OpenClaw is installed through its official Windows path instead of assuming npm is the only option.

- AgenticBoot detects whether `openclaw` is already usable before installing.
- If OpenClaw is missing, Windows uses the official PowerShell installer path.
- If OpenClaw already exists outside the managed root, AgenticBoot treats it as installed and skips reinstalling it.

## Ownership and uninstall

- AgenticBoot only claims ownership over installations it created under the managed install root.
- External OpenClaw installs are not auto-deleted by uninstall.

## Manual verification

```powershell
openclaw --version
```
