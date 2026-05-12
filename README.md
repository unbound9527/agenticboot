# AgenticBoot

AgenticBoot is a Windows-first bootstrapper for AI coding tools. It detects what is already installed, skips redundant work, installs the tools you select, and keeps uninstall behavior safe for anything it does not own.

This branch currently focuses on real Windows install flows. macOS and Linux scaffolding exists, but those install paths are not fully implemented yet.

## What works on Windows

- Detect existing `Node.js`, `Git`, and supported AI tools before installing.
- Reuse a working local installation instead of forcing a reinstall.
- Install official desktop apps for:
  - Claude Desktop
  - Codex desktop app
  - OpenCode desktop app
- Install CLI tools for:
  - Claude Code
  - Codex CLI
  - Gemini CLI
  - OpenCode CLI
  - OpenClaw
  - Hermes
- Install Hermes without requiring preinstalled Python by downloading a managed Python runtime into the Hermes tool directory.
- Only remove AgenticBoot-managed files automatically during uninstall.

## Key behavior

### Detection-first install

AgenticBoot checks whether a tool is already usable on the machine before installing it. That applies to dependencies and user-facing tools.

- If `Node.js` or `Git` already works, AgenticBoot reuses it.
- If a CLI tool is already available, AgenticBoot skips reinstalling it.
- If a desktop app is already installed outside the managed root, AgenticBoot treats it as installed and does not pretend it can safely uninstall it.

### Managed vs external installs

AgenticBoot distinguishes between:

- Managed installs: files created under the selected install root.
- External installs: tools already installed elsewhere on the system.

Only managed installs are candidates for automatic directory cleanup during uninstall.

## Tool notes

- OpenCode CLI on Windows uses the native `opencode-ai` npm package. It does not depend on WSL.
- Hermes on Windows uses an AgenticBoot-managed official Python 3.11 ZIP runtime plus a local `venv`, installing Hermes Agent from the published GitHub source archive (tiered `pip install`, matching upstream Windows guidance). It does not require the user to preinstall Python, Git, or rely on `winget`.
- OpenClaw on Windows uses the official PowerShell install path.

## Current platform status

- Windows: implemented
- macOS: framework only
- Linux: framework only

## Development startup

On Windows, use the repository-managed startup script to run the desktop app with the Node.js version declared in [`.node-version`](./.node-version) and the `pnpm` version declared in [`package.json`](./package.json).

```powershell
.\scripts\dev-desktop.ps1
```

You can also use the `cmd` wrapper:

```bat
scripts\dev-desktop.cmd
```

The script downloads a managed Node.js runtime on first use, runs `pnpm install --frozen-lockfile` through Corepack, and then starts the Tauri desktop app.

## Repo docs

- Tool docs: [docs/tools/README.md](./docs/tools/README.md)
- Windows install design: [docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md](./docs/superpowers/specs/2026-05-08-windows-one-click-install-design.md)
- Implementation plan: [docs/superpowers/plans/2026-05-08-windows-one-click-install.md](./docs/superpowers/plans/2026-05-08-windows-one-click-install.md)
