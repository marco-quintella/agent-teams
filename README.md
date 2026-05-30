# claude-orchestrator

Local control plane for Claude Code agent teams with a web kanban UI.

## Status

V1 control plane + **V1.1** session hardening + **V1.2** self-contained localhost + **V1.3** operator workspace (team history, folder browse, default model):

- `crates/orchestrator-core` — domain, SQLite, supervisor, ATOP ingestor, Claude settings
- `crates/orchestrator-server` — REST API, WebSocket, `serve` command
- `web/` — Svelte 5 + Vite kanban UI

## Prerequisites

- Rust toolchain (2021 edition)
- Node.js 20+ (for building `web/`)
- On Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** (`link.exe`)
- Claude Code CLI on `PATH` for real agent spawn (`claude` or `claude-code`)

## Build

```bash
cargo check --workspace
cargo test --workspace
cd web && npm install && npm run build
```

## Development (local) — V1.2 single process

One command builds the UI and serves API + SPA on **http://127.0.0.1:47821**:

```powershell
.\scripts\dev.ps1
```

```bash
./scripts/dev.sh
```

Open that URL → **Settings** to configure Claude (CLI login or API key, optional OpenRouter base URL) → **Board** to launch a team.

### Advanced: Vite hot reload

If you want HMR while editing the UI:

```bash
set ORCHESTRATOR_UI=vite
# terminal A: cargo run -p orchestrator-server -- serve
# terminal B: cd web && npm run dev  → http://localhost:5173
```

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

### Claude auth in container

Mount or inject Claude credentials the same way you would for Claude Code in Docker (e.g. config under the container user home), or use **Settings → API key** with env-compatible keys. V1.2 stores API keys encrypted under `.data/` on the host.

## Security

**V1.2 still has no HTTP API authentication** for the orchestrator itself. Bind to loopback in dev. For production, put the service behind a reverse proxy with TLS and auth before exposing it on a VPS.

API keys saved in Settings are encrypted at rest (local `.data/orchestrator.key` + SQLite). Do not commit `.data/`.

## V1.3 operator workspace (localhost)

- **Saved teams** — Board sidebar lists teams from SQLite; select one and **Launch** / **Stop** without re-entering config.
- **Browse…** — When creating a team, pick a project folder via native dialog (or type the path).
- **Default model** — Settings → default model (`sonnet`, `opus`, `haiku`, custom, or CLI default); passed as `--model` to each `claude` session on launch.

### V1.3 manual checks

1. Create a team, reload the page — team appears in **Saved teams**; select and launch.
2. **Browse…** fills project path (requires desktop display; headless/CI falls back to typed path).
3. Set model in Settings, launch — verify `claude` argv includes `--model` (maintainer machine).

## V1.2 manual acceptance (localhost)

1. `.\scripts\dev.ps1` (or build web + `cargo run -p orchestrator-server -- serve` with `web/dist` present).
2. Open **http://127.0.0.1:47821** → **Settings** → configure CLI login or API key; doctor shows **ready**.
3. **Board** → create project + team (lead + worker), **Launch**.
4. Send an **explicit** objective to the lead (e.g. “Create a backlog task titled ‘CONTRIBUTING setup’ for adding install steps to CONTRIBUTING.md”).
5. Within ~15 minutes, lead snippet shows activity and optionally a `created_by=agent` card (lead decides via ATOP — orchestrator does not write protocol).
6. Sending “hello” alone is **not** a failure if no task appears (AE4 negative control).
7. **Stop** team — sessions end; relaunch works without pipe-closed false success.

## OpenRouter (API key mode)

In Settings, choose **API key**, paste your OpenRouter key, set base URL to `https://openrouter.ai/api`. The orchestrator passes `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL` to spawned `claude` children. Verify with your installed Claude Code version.

## Known limitations

- No mailbox, code review UI, or multi-provider agent runtimes
- **No orchestrator HTTP auth** — network exposure requires a reverse proxy
- Lead-driven kanban updates depend on the lead using ATOP when appropriate
- Orchestrator restart does not resurrect running agents
- On Windows, stop `orchestrator-server` before `cargo build` if the exe is locked

## Docs

- Requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md`
- V1.1 requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.1-requirements.md`
- V1.2 requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.2-requirements.md`
- V1.3 requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.3-requirements.md`
- Plans: `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`, `docs/plans/2026-05-30-002-feat-agent-orchestrator-v1.1-plan.md`, `docs/plans/2026-05-30-003-feat-agent-orchestrator-v1.2-plan.md`, `docs/plans/2026-05-30-004-feat-agent-orchestrator-v1.3-plan.md`
- Architecture: `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
- V1.2 operator guide (Settings, single-serve, lead autonomy): `docs/solutions/developer-experience/orchestrator-v1.2-self-contained-localhost.md`
- V1.1 troubleshooting (API hang / PTY): `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`
- V1.3 troubleshooting (Svelte UI freeze, browse in headless): `docs/solutions/ui-bugs/orchestrator-v1.3-svelte-effect-loop-and-launcher-remount.md`
