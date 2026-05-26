# OpenCode

## Windows support

OpenCode is supported on Windows in the current AgenticBoot implementation.

### OpenCode CLI

- Windows downloads the published `opencode-ai` package artifact and installs the bundled `opencode.exe` directly under the managed root.
- AgenticBoot detects an existing `opencode` command first and skips reinstalling it when already usable.
- This path does not require WSL or a preinstalled Node.js runtime.

### OpenCode desktop app

- Windows desktop installs use the official desktop app flow.
- AgenticBoot detects an existing desktop install before attempting to install again.
- External desktop installs are shown as installed but are not treated as safe uninstall targets.

## What AgenticBoot owns

If AgenticBoot installs a managed CLI copy under the selected install root, it can manage that copy. If OpenCode is already installed elsewhere, AgenticBoot reuses it without claiming ownership.
