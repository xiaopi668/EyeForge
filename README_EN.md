# EyeForge

A fully Rust-native AI desktop assistant and gateway.

Current entry points:
- Desktop app: `src-rs/target/debug/eye-forge-rs.exe`
- Web UI: `http://127.0.0.1:9178/`
- WebSocket: `ws://127.0.0.1:9178/ws`

## Current State

- The Python runtime backend has been removed
- The project now runs on a Rust-native execution chain
- The Rust desktop app owns config, task input, execution logs, and gateway status
- The Rust gateway serves both the Web UI and the WebSocket API on port `9178`
- The Rust runtime currently supports:
  - `shell`
  - `wait`
  - `open`
  - `click`
  - `type`
  - `hotkey`
  - `scroll`
  - `screenshot`
  - `complete`
- Natural language tasks can now go through a Rust-native LLM planning path and then execute as action queues

## Quick Start

Install Rust first:
- [https://rustup.rs/](https://rustup.rs/)

### Build

```powershell
install.bat
```

### Run

```powershell
start.bat
```

After launch:
- The desktop window opens
- The gateway listens on `http://127.0.0.1:9178/`

## WebSocket Protocol

Endpoint:
- `ws://127.0.0.1:9178/ws`

Flow:
1. `auth`
2. `task`
3. `result`

Example:

```json
{
  "type": "task",
  "task": "{\"actions\":[{\"type\":\"wait\",\"seconds\":0.5},{\"type\":\"complete\",\"result\":\"ok\"}]}"
}
```

## Repository Layout

```text
EyeForge/
├── src-rs/               # Rust desktop app, gateway, runtime
├── web-ui/               # Browser UI served by the Rust gateway
├── docs/                 # Documentation
├── install.bat           # Rust build script
├── start.bat             # Rust launcher
├── setup_stub.cs         # Installer stub
├── CHANGELOG_EN.md
└── README.md
```

## Notes

The project is now detached from the Python backend, but not every historical feature has been ported 1:1 yet.

Still being migrated or expanded:
- richer LLM planning
- vision understanding
- broader desktop automation actions
- voice / wake-word support
- multi-platform channel bridges
