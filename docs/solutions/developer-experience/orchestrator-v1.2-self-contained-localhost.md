---
title: "Orchestrator V1.2 — self-contained localhost operator loop"
date: 2026-05-30
category: developer-experience
module: claude-orchestrator
problem_type: developer_experience
component: development_workflow
severity: medium
applies_when:
  - "Shipping or operating claude-orchestrator on localhost without a separate Claude Code app window"
  - "Operators need in-product Claude CLI install, credentials, doctor, and launch from one URL"
  - "Lead must decide kanban actions via ATOP without orchestrator writing protocol lines"
tags:
  - orchestrator
  - v1-2
  - localhost
  - settings
  - credentials
  - single-serve
  - claude-cli
  - atop
related_components:
  - orchestrator-core
  - orchestrator-server
  - web
---

# Orchestrator V1.2 — self-contained localhost operator loop

## Context

Through V1.1, the control plane could spawn persistent `claude` PTY sessions and deliver lead objectives, but operators still ran **API + Vite in two terminals**, configured Claude **outside** the product, and hit confusing failures when child processes died (pipe closed, hung UI). V1.1 also risked treating “lead created a task” as a deterministic orchestrator outcome rather than **lead judgment**.

V1.2 closes the **operator autonomy** gap: one process serves API and built UI, Settings owns Claude credentials, launch is gated on doctor readiness, and the lead remains the only writer of ATOP protocol lines.

## Guidance

### Single-process localhost (default dev loop)

- Build the UI once (`web/dist`), then run only `orchestrator-server serve` on port **47821**.
- When `web/dist/index.html` exists, the server mounts the SPA in **dev and prod** profiles (unless `ORCHESTRATOR_UI=vite`).
- Helper: `scripts/dev.ps1` / `scripts/dev.sh` — build web, then serve.
- Open **http://127.0.0.1:47821** for Board and Settings (no separate Vite terminal required).

### Claude credentials (Settings scope only)

- **SQLite** table `claude_settings` (migration `002_claude_settings.sql`), singleton row `default`.
- **API key mode:** ChaCha20-Poly1305 ciphertext in DB; master key in `.data/orchestrator.key` (gitignored). Spawn injects `ANTHROPIC_API_KEY` and optional `ANTHROPIC_BASE_URL` (OpenRouter: `https://openrouter.ai/api`).
- **CLI login mode:** marker under `~/.claude/` (e.g. `.credentials.json`); guided `claude login` via `POST /api/setup/claude-login`.
- REST: `GET/PATCH /api/setup/claude-settings`, `GET /api/setup/doctor`, `POST /api/setup/install-claude` (requires `{ "confirm": true }`).
- **Launch** fails with a clear error if CLI missing or credentials not ready (`CredentialsNotConfigured`).

### Session reliability (carry-forward from V1.1 fixes)

- All PTY spawn, bootstrap stdin, and lead message delivery stay in `tokio::task::spawn_blocking` (see `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`).
- Snippet refresh detects dead children (`MemberSession::is_alive`); agent run → `error`, session removed from map.
- `POST /teams/{id}/message` returns **409** when lead PTY is gone (not a generic 400).
- `stop_all_sessions()` before relaunch to avoid zombie `claude` processes.

### Lead autonomy (product rule)

- Objectives are **conversational**; validation uses explicit asks (e.g. “create a backlog task for …”), not trivial pings.
- **Orchestrator never** appends `protocol.ndjson` or creates `created_by=agent` tasks to force acceptance.
- Lead role/bootstrap copy uses conditional ATOP language (`crates/orchestrator-core/src/supervisor/bootstrap.rs`, `resources/atop-v1.md`).
- Reject deterministic “relay” (`claude -p` shadow turns writing protocol) — out of product identity.

### Explicit non-goals in V1.2

- Orchestrator **HTTP/API token** auth (LAN/VPS) — deferred.
- Mandatory Docker-compose acceptance gate — Docker remains production path only.

## Why This Matters

Without in-product credentials and a single URL, every operator repeats manual CLI setup and misattributes orchestrator bugs to “Claude not working.” Without lead autonomy, the team either cheats acceptance with protocol relays or ships a control plane that overrides agent judgment—both erode trust in the kanban loop.

## When to Apply

- Implementing or debugging V1.2+ localhost flows (Settings, doctor, launch, message).
- Onboarding a new machine: run `dev.ps1`, open Settings, then Board.
- When message delivery fails: check doctor, agent run status (`error`), and that `claude` children are alive before blaming ATOP.

## Examples

**Before (V1.1 typical localhost):**

```text
Terminal A: cargo run -p orchestrator-server -- serve
Terminal B: cd web && npm run dev
Claude: claude login manually; API key in shell env
```

**After (V1.2):**

```powershell
.\scripts\dev.ps1
# → http://127.0.0.1:47821
# Settings → API key or CLI login → doctor green → Board → Launch
```

**OpenRouter (api_key mode):**

- Settings: paste key, base URL `https://openrouter.ai/api`, Save.
- Launch spawns children with `ANTHROPIC_API_KEY` + `ANTHROPIC_BASE_URL`.

**Acceptance objective (AE3-style):**

- Send: *“Create a backlog task titled ‘CONTRIBUTING setup’ for adding install steps to CONTRIBUTING.md.”*
- Success: lead activity in snippet and/or lead-authored card—not orchestrator-written protocol.

## Related

- Architecture index: `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md` (V1.2 delta summary)
- PTY/Tokio blocking and pipe-closed UX: `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`
- V1.1 requirements (PTY objectives baseline): `docs/brainstorms/2026-05-30-agent-orchestrator-v1.1-requirements.md`
- V1.1 plan: `docs/plans/2026-05-30-002-feat-agent-orchestrator-v1.1-plan.md`
- V1.2 requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.2-requirements.md`
- V1.2 plan: `docs/plans/2026-05-30-003-feat-agent-orchestrator-v1.2-plan.md`
- README: local quickstart and AE1–AE7 checklist
