---
title: Orchestrator API hangs when PTY spawn runs on the Tokio runtime
date: 2026-05-30
category: performance-issues
module: claude-orchestrator
problem_type: performance_issue
component: development_workflow
symptoms:
  - "POST /api/teams/{id}/launch and POST /message appear to hang; health checks time out"
  - "UI Launch button stays disabled (busy) while agents may already show running in the API"
  - "Windows rebuild fails with access denied on orchestrator-server.exe while the old server is still running"
root_cause: thread_violation
resolution_type: code_fix
severity: high
tags:
  - rust
  - tokio
  - pty
  - orchestrator
  - spawn-blocking
  - windows
related_components:
  - orchestrator-core
  - orchestrator-server
  - web
---

# Orchestrator API hangs when PTY spawn runs on the Tokio runtime

## Problem

During V1.1 browser QA, launching a team or sending a message could freeze the Axum server: `/api/health` stopped responding, the Svelte UI stayed in a global `busy` state, and operators could not tell whether launch succeeded. The root issue was **synchronous PTY I/O on the async executor**, compounded by UI state that treated all actions as one busy flag.

## Symptoms

- `curl` / `Invoke-WebRequest` to `/api/health` hangs until the server process is killed or launch completes.
- Launch eventually creates `running` agent runs in SQLite, but the browser never clears `busy` or shows agents until a full reload.
- Rebuilding while `orchestrator-server.exe` is running on Windows: `error: failed to remove file ... orchestrator-server.exe` (access denied).
- After fix, `POST /message` may return **400** with *"O pipe está sendo fechado"* when Claude child processes have already exited (PTY writer closed).

## What Didn't Work

- Restarting only the Vite dev server — the API binary still blocked the runtime.
- Assuming the UI kanban bug was primary — tasks existed in the API but the UI was stale because `loadTeam` ran before launch finished or never completed after a hung request.
- Using `browser_fill` without blur for Svelte `bind:value` — Launch stayed disabled until a blur event updated `projectPath`.

## Solution

### 1. Move blocking PTY work off the async runtime

**Launch path** (`crates/orchestrator-core/src/supervisor/mod.rs`): wrap `MemberSession::spawn` and bootstrap `write_stdin` in `tokio::task::spawn_blocking`:

```rust
let session = tokio::task::spawn_blocking(move || {
    let session = MemberSession::spawn(
        &project_root, &team_id, &member_id, &cmd_path, &args, &role_md,
    )?;
    session.write_stdin(&bootstrap)?;
    Ok::<Arc<MemberSession>, anyhow::Error>(Arc::new(session))
})
.await
.map_err(|e| anyhow::anyhow!("spawn task join failed: {e}"))??;
```

**Message path** (`crates/orchestrator-server/src/app_state.rs`): already uses `spawn_blocking` for `deliver_lead_message` — keep both paths consistent.

`MemberSession::spawn` opens a PTY, spawns `claude`, starts a reader thread, and writes role files — all **blocking** and unsuitable for direct `.await` chains on Tokio worker threads.

### 2. Stop orphaned sessions before relaunch

`launch_team` calls `supervisor.stop_all_sessions()` so a new launch does not leave zombie Claude processes or stale DashMap entries.

### 3. UI: separate busy flags and safer `loadTeam`

`web/src/lib/components/TeamLauncher.svelte`:

- `launchBusy` vs `messageBusy` so Send does not disable Launch/Stop.
- `loadTeam` sets `teamId` / `localStorage` only **after** parallel fetches succeed.
- `createAndLaunch` attempts `loadTeam` on partial failure if the team id was created.

`web/src/lib/stores/orchestrator.ts`: on resume failure, clear `teamId`, tasks, members, and `agentRuns`.

### 4. Operational: restart the API on Windows

```powershell
Get-Process -Name orchestrator-server -ErrorAction SilentlyContinue | Stop-Process -Force
cargo run -p orchestrator-server -- serve
```

Rebuild while the exe is locked produces access denied (os error 5).

## Why This Works

Tokio’s default multi-thread runtime expects async tasks not to block for seconds. PTY spawn + process start + stdin write can block indefinitely. When enough worker threads stall, **all** HTTP handlers (including `/api/health`) queue behind blocked work.

`spawn_blocking` runs the closure on a dedicated blocking thread pool, keeping accept/readiness on async workers.

## Prevention

- **Any** supervisor call that touches PTY, `std::process`, or joins threads should run in `spawn_blocking` (or a dedicated blocking thread), not inside async handlers.
- After changing Rust server code, **restart** `orchestrator-server`; HMR does not reload the binary.
- On Windows, stop the server before `cargo run` / `cargo build` if the exe is in use.
- For UI automation: type into bound fields and blur, or use `browser_type`, so Svelte 5 runes update state.
- Message delivery requires a **live** lead PTY; verify `Get-Process claude*` (or equivalent) after launch. Pipe-closed errors are expected if the child exited.
- Manual acceptance: launch → add task (card in Backlog) → send message → confirm `[orchestrator-objective]` in `.orchestrator/teams/{team_id}/{lead_member_id}/inbound.md`.

## Related Issues

- Architecture overview: `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
- V1.2 operator loop (Settings, single-serve, session health): `docs/solutions/developer-experience/orchestrator-v1.2-self-contained-localhost.md`
- Requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.1-requirements.md`
- Plan: `docs/plans/2026-05-30-002-feat-agent-orchestrator-v1.1-plan.md`
- README: `README.md` (Docs section)
