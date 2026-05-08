# Node.js

## How AgenticBoot handles Node.js on Windows

AgenticBoot treats Node.js as a dependency and checks for it before installing anything that needs npm.

- If a working `node` is already available, AgenticBoot reuses it.
- If Node.js is missing, AgenticBoot installs a managed copy under the selected install root.
- Managed Node.js installs are isolated from the rest of the system.

## What this means for users

- You do not need to uninstall your existing Node.js just to use AgenticBoot.
- You do not need to preinstall Node.js if it is missing.
- AgenticBoot should only remove the managed copy it created itself.

## Manual verification

```powershell
node --version
npm --version
```
