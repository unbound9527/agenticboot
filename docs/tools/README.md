# Tool Notes

This directory contains concise notes about how AgenticBoot currently handles supported tools.

## Windows-focused behavior

The current implementation is Windows-first:

- Existing installations are detected before install.
- Dependencies such as `Node.js` and `Git` are reused if already present.
- Official desktop apps are installed as desktop apps, not replaced with npm stand-ins.
- Hermes Desktop follows the official desktop installer path on Windows and does not require the user to preinstall Python.

## Documents

- [nodejs.md](./nodejs.md)
- [git.md](./git.md)
- [opencode.md](./opencode.md)
- [openclaw.md](./openclaw.md)
