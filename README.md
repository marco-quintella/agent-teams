# claude-orchestrator

Local control plane for Claude Code agent teams with a web kanban UI.

## Status

V1 in progress. Current workspace:

- `crates/orchestrator-core` — domain, agents, SQLite store
- `crates/orchestrator-server` — CLI (`orchestrator-server`)

## Prerequisites

- Rust toolchain (2021 edition)
- On Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** (provides `link.exe`)
- Claude Code CLI on `PATH` for agent spawn (later units)

## Build

```bash
cargo check --workspace
cargo test -p orchestrator-core
```

## CLI (interim)

```bash
cargo run -p orchestrator-server -- validate examples/basic-workflow.yaml
cargo run -p orchestrator-server -- doctor
```

## Deployment (planned)

- **Development:** localhost — Vite + API on loopback (see plan)
- **Production:** Docker on VPS — single container serves API + built Svelte UI

See `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`.
