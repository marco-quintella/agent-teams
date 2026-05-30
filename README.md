# claude-orchestrator

Local control plane for Claude Code agent teams with a web kanban UI.

## Status

V1 control plane + **V1.1 session hardening** (interactive Claude per teammate, lead objectives via PTY, ATOP proof path):

- `crates/orchestrator-core` — domain, SQLite, supervisor, ATOP ingestor
- `crates/orchestrator-server` — REST API, WebSocket, `serve` command
- `web/` — Svelte 5 + Vite kanban UI

## Prerequisites

- Rust toolchain (2021 edition)
- Node.js 20+ (for `web/`)
- On Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** (`link.exe`)
- Claude Code CLI on `PATH` for real agent spawn (`claude` or `claude-code`)

## Build

```bash
cargo check --workspace
cargo test --workspace
cd web && npm install && npm run build
```

## Development (local)

Terminal A — API (loopback, port **47821**):

```bash
set ORCHESTRATOR_PROFILE=dev
set ORCHESTRATOR_PORT=47821
cargo run -p orchestrator-server -- serve
```

Terminal B — UI (proxies `/api` and `/ws` to the API):

```bash
cd web
npm install
npm run dev
```

Open **http://localhost:5173**. Or use the helper script:

```powershell
.\scripts\dev.ps1
```

```bash
./scripts/dev.sh
```

Copy `.env.example` to `.env` and adjust paths as needed.

## CLI

```bash
cargo run -p orchestrator-server -- doctor
cargo run -p orchestrator-server -- validate examples/basic-workflow.yaml
cargo run -p orchestrator-server -- serve
```

## Deploy on VPS (Docker)

```bash
docker compose -f docker/docker-compose.yml up --build
```

UI and API: **http://localhost:47821** (map the port on your VPS firewall as needed).

Volumes:

- `orchestrator-data` — SQLite database
- `./workspace` — mount your project repos for agent workspaces

### Claude auth in container

Mount or inject Claude credentials the same way you would for Claude Code in Docker (e.g. config under the container user home). V1 does not automate OAuth; ensure `claude` is available in the image if you need live agents.

## Security

**V1 has no authentication.** Bind to loopback in dev. For production, put the service behind a reverse proxy with TLS and auth before exposing it on a VPS.

## V1.1 manual acceptance (localhost)

1. `cargo run -p orchestrator-server -- doctor` — Claude on PATH and version string.
2. Start API + UI (`scripts/dev.ps1` or two terminals per **Development** above).
3. Create project + team (lead + at least one worker), **Launch**.
4. Send an objective to the lead (e.g. “Add a CONTRIBUTING.md with install steps”).
5. Within ~5 minutes, confirm a new kanban card with `created_by=agent` (lead appended `task.create` to `protocol.ndjson`).
6. **Stop** team — all sessions end within ~15s.

Debug ATOP without Claude: append a line to `.orchestrator/teams/{team_id}/{member_id}/protocol.ndjson` per `crates/orchestrator-core/resources/atop-v1.md`.

## Known limitations

- No mailbox, code review UI, or multi-provider support
- **No API authentication** — do not expose to the public internet without auth (V1.2+)
- Workers launch real sessions but V1.1 proof is **lead creates task** only
- ATOP adherence depends on Claude following role.md; use debug `echo` to protocol file if needed

## Docs

- Requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md`
- V1.1 requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.1-requirements.md`
- Plan: `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`
- V1.1 plan: `docs/plans/2026-05-30-002-feat-agent-orchestrator-v1.1-plan.md`
- Architecture: `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
- V1.1 troubleshooting (API hang / PTY blocking): `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`
