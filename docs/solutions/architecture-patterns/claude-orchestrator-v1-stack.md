---
title: Claude orchestrator V1 — Rust control plane, ATOP, and Svelte UI
date: 2026-05-30
category: architecture-patterns
module: claude-orchestrator
problem_type: architecture_pattern
component: development_workflow
severity: medium
applies_when:
  - "Building a local or Docker-deployed control plane for multiple Claude Code CLI teammates"
  - "Needing a kanban board with human and agent task updates over WebSocket"
  - "Starting from a non-compiling YAML workflow stub and shipping a full V1 stack"
tags:
  - rust
  - svelte
  - sqlite
  - orchestrator
  - atop
  - vite
  - docker
related_components:
  - orchestrator-core
  - orchestrator-server
  - web
---

# Claude orchestrator V1 — Rust control plane, ATOP, and Svelte UI

## Context

The repository began as a **non-compiling** YAML workflow sketch with a stubbed `Orchestrator::execute`. The goal was V1 of **claude-orchestrator**: a control plane where an operator creates teams, launches **Claude Code CLI** processes per member, and manages a **real-time kanban** with mixed human/agent updates—matching the operator's existing workaround (parallel Claude Code via scripts) but with centralized spawn/stop and shared task state.

Work was executed on branch `feat/orchestrator-v1`, split into **six atomic commits**, merged to `master` via fast-forward. Requirements and plan live in `docs/brainstorms/` and `docs/plans/`; this doc captures the **implemented architecture** and operational lessons.

## Guidance

### Workspace layout

| Piece | Role |
|-------|------|
| `crates/orchestrator-core` | Domain types, SQLite `Store`, PTY **supervisor**, **ATOP** ingestor, in-process **EventBus** |
| `crates/orchestrator-server` | Thin binary + lib: Axum REST, WebSocket, `serve` subcommand, static SPA in prod |
| `web/` | Svelte 5 + Vite UI: team launcher, kanban (dnd), agent status panel |

SQLite is the source of truth (`sqlx` migrations in `crates/orchestrator-core/migrations/001_initial.sql`). Runtime protocol files live under **`.orchestrator/teams/{team_id}/{member_id}/`** on the project root (gitignored).

### Runtime profiles (KTD10)

| Variable | Dev | Prod |
|----------|-----|------|
| `ORCHESTRATOR_PROFILE` | `dev` | `prod` |
| Bind | `127.0.0.1:47821` | `0.0.0.0:47821` |
| UI | Vite `:5173` proxies `/api` and `/ws` | `ORCHESTRATOR_STATIC_DIR` serves `web/dist` |
| CORS | Permissive for Vite origins | Not applied |

Default API port is **47821** (not 8080).

### ATOP v1 (agent → orchestrator)

Agents mutate tasks only by appending **one JSON object per line** to `protocol.ndjson`. Ops: `task.create`, `task.update_status`, `task.assign`, `ping`. Spec: `crates/orchestrator-core/resources/atop-v1.md`.

Serde requires explicit renames for dotted op names:

```rust
#[serde(tag = "op")]
pub enum AtopMessage {
    #[serde(rename = "task.create")]
    TaskCreate { /* ... */ },
}
```

`AtopIngestor` tails the file on a 500ms loop and applies DB changes with `TaskActor::Agent`, publishing `OrchestratorEvent::TaskUpdated` on the bus.

### Supervisor and sessions

- **Supervisor** spawns children via `portable-pty`; mock command (`cmd /C echo` on Windows) used in tests.
- **V1 placeholder:** real launch uses `claude --version` until long-running CLI flags are verified.
- Output ring buffer: last **2048** bytes in `MemberSession` (`std::sync::Mutex` in reader thread—do not use `tokio::RwLock` from blocking PTY reader).
- Lead operator messages: `deliver_lead_message` appends to `inbound.md` (PTY writer not retained in V1).

### HTTP API surface

REST under `/api`:

- `POST /projects`, `POST /teams`, `POST /teams/{id}/members`
- `POST /teams/{id}/launch`, `POST /teams/{id}/stop`, `POST /teams/{id}/message`
- `GET /teams/{id}/tasks`, `POST`/`PATCH` tasks
- `GET /teams/{id}/members`, `GET /teams/{id}/agent-runs`
- `GET /health`

WebSocket: `/ws` — JSON events `task_updated`, `agent_run_updated`, `team_updated`.

Launch is **idempotent-guarded**: second launch returns **409** if sessions already exist. Project paths containing `..` return **400**.

### Web UI

- API client: `web/src/lib/api/client.ts` — `VITE_API_BASE` empty in prod (same origin).
- State: Svelte stores in `web/src/lib/stores/orchestrator.ts` + WS merge.
- Kanban: `svelte-dnd-action`; PATCH on column drop.

### Commits and merge

Use **atomic commits** per layer (see `AGENTS.md` and `.cursor/skills/atomic-commits/SKILL.md`):

1. `build:` workspace deps  
2. `feat(core):` supervisor, events, ATOP  
3. `feat(server):` API + WS  
4. `feat(web):` Svelte UI  
5. `build:` Docker + dev scripts  
6. `docs:` README  

Merge feature branch with `git checkout master && git merge feat/...` (fast-forward when linear).

### Local dev

```powershell
$env:ORCHESTRATOR_PROFILE="dev"
cargo run -p orchestrator-server -- serve
# separate terminal
cd web && npm run dev
```

Or `.\scripts\dev.ps1` / `./scripts/dev.sh`.

### Tests that must pass before shipping

```bash
cargo test --workspace
cd web && npm run build
```

## Why This Matters

Without this split, UI, protocol, and persistence get tangled in one crate—hard to test ATOP parsing or SQLite without spinning HTTP. ATOP keeps agent task mutations **deterministic** instead of parsing chat. EventBus decouples ingest from WebSocket subscribers so REST handlers and ATOP share one notification path.

**Security:** V1 has **no auth**. Do not expose prod bind `0.0.0.0` on a VPS without a reverse proxy and authentication.

## When to Apply

- Adding a new agent capability → extend ATOP schema + ingestor + spec markdown; add ingest test in `crates/orchestrator-core/tests/atop_test.rs`.
- New REST endpoints → `orchestrator-server/src/routes/`, import `orchestrator_core::Store` trait in handlers using `Arc<SqliteStore>`.
- UI features → `web/src/lib/api/client.ts` + components; verify WS event types match `OrchestratorEvent` serde names (`snake_case` tag).
- **Windows dev:** install Visual Studio Build Tools with **Desktop development with C++** so `link.exe` exists before `cargo build`.

## Examples

**Windows compile failure (environment):**

```
error: linker `link.exe` not found
```

Fix: install MSVC Build Tools; re-run `cargo check --workspace`.

**Integration test pattern for API (no nested runtime):**

```rust
async fn test_state(dir: &tempfile::TempDir) -> AppState {
    let store = SqliteStore::connect(&db_url).await.unwrap();
    // ...
}

#[tokio::test]
async fn create_task_emits_ws_payload_shape() {
    let state = test_state(&dir).await; // await, do not block_on inside tokio test
}
```

**Docker prod:**

```bash
docker compose -f docker/docker-compose.yml up --build
# UI + API at http://localhost:47821
```

## Related

- Requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md`
- Plan: `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`
- ATOP spec: `crates/orchestrator-core/resources/atop-v1.md`

## V1 gaps (intentional)

- Persistent `claude` session invocation not finalized (`--version` placeholder)
- `task.assign` via ATOP works; PTY stdin not wired for live lead chat
- No auth/TLS, mailbox, or multi-provider
