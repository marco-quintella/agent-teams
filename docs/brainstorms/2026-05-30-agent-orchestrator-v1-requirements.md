---
title: "Agent Orchestrator V1"
status: active
date: 2026-05-30
type: requirements
origin_conversation: ce-brainstorm 2026-05-30
---

## Summary

V1 delivers an **agent-team control plane** with a **web UI**: create a team bound to a project directory, launch and stop **Claude Code CLI** teammates, and run a **real-time kanban** where humans and agents create and move tasks.

**Deployment target:** run in **Docker on a VPS**, controlled remotely via the browser. **Development** runs the same stack **locally** (no Docker required day-to-day). The product replaces ad-hoc parallel Claude Code scripts with centralized lifecycle and visibility—not parity with Agent Teams AI desktop.

---

## Problem Frame

Teams already run **multiple Claude Code sessions in parallel via scripts**, but lack a single place to see status, assign work, and coordinate task state. V1 proves that a **Rust orchestrator + local web UI** can own process lifecycle and task board state while staying Claude-only.

---

## Actors

- **A1 — Operator (human):** Creates teams, launches agents, creates tasks, moves cards, sends objectives to the lead.
- **A2 — Lead agent:** Claude Code session with lead role; may create and update tasks per orchestrator protocol.
- **A3 — Worker agent:** Claude Code session with worker role; executes assigned tasks; may update task status.
- **A4 — Orchestrator (system):** Persists state, supervises subprocesses, parses agent task actions, broadcasts events to UI.

---

## Key Flows

- **F1 — Bootstrap workspace:** Operator registers a local project path; orchestrator validates it exists and is writable for state sidecars.
- **F2 — Create and launch team:** Operator defines team name, roles (lead + workers), provisioning prompt; orchestrator spawns one Claude Code process per member with role context.
- **F3 — Mixed task lifecycle:** Operator or agents create tasks; assign to member; move across kanban columns; orchestrator persists and pushes WebSocket updates.
- **F4 — Observe and intervene:** Operator views agent status (running/idle/error) and minimal last output; stops or restarts a teammate; sends a high-level message to the lead.
- **F5 — Shutdown:** Operator stops team; orchestrator terminates child processes and marks agents stopped.

---

## Requirements

- **R1.** Operator can bind the orchestrator to a single local **project root** per session.
- **R2.** Operator can define a **team** with at least one **lead** and zero or more **workers**, each with a display name and role prompt fragment.
- **R3.** Operator can **launch** all team members as **Claude Code CLI** child processes with `cwd` set to the project root and role-specific system context.
- **R4.** Operator can **stop** or **restart** an individual teammate or the whole team.
- **R5.** The system persists **teams, members, tasks, and status history** across orchestrator restarts (local store).
- **R6.** Operator can view a **kanban board** with fixed columns: **Backlog**, **In Progress**, **Review**, **Done**.
- **R7.** Operator can **create**, **edit** (title/description), **assign**, and **drag** tasks between columns.
- **R8.** Agents can **create tasks** and **change task status/assignee** only through an **orchestrator-defined machine protocol** (not free-form chat parsing).
- **R9.** The UI updates **in real time** when task or agent state changes (target &lt; 2s perceived latency on dev machine or VPS over a normal network).
- **R10.** Operator sees per-agent **status** (starting, running, idle, error, stopped) and a **short last-output snippet** (not full session analytics).
- **R11.** Operator can send a **single team objective message** to the lead (text); orchestrator delivers it into the lead session input channel.
- **R12.** V1 supports **Claude Code CLI only**; no Codex/OpenCode/API-only agents.
- **R13.** **Development:** operator uses the web UI on **localhost** (Vite dev server + Rust API on loopback).
- **R14.** Multiple agents on the **same project root** without worktree isolation are supported with a **visible warning** about Git conflict risk.
- **R15.** **Production:** operator can run the orchestrator in **Docker on a VPS** and use the **same web UI** from a remote browser; API and UI are served from one deployable unit (no separate frontend host required).
- **R16.** **Production data** (SQLite, `.orchestrator/` sidecars) lives on **mounted volumes** so container restarts do not wipe teams/tasks.

---

## Acceptance Examples

- **AE1.** Covers R2, R3, F2 — **Given** a valid project path and installed `claude` CLI, **When** operator creates a team with lead "lead" and worker "impl" and clicks Launch, **Then** two child processes start, both appear as `running` or `starting` in the UI within 30s.
- **AE2.** Covers R6, R7, F3 — **Given** a running team, **When** operator creates task "Add README" in Backlog and drags to In Progress assigned to `impl`, **Then** the card appears on the board for all connected UI clients without refresh.
- **AE3.** Covers R8, F3 — **Given** worker `impl` emits a valid protocol message to create task "Fix tests", **When** the orchestrator ingests it, **Then** a new card appears in Backlog with `created_by=agent`.
- **AE4.** Covers R4, F5 — **When** operator stops the team, **Then** all child processes terminate within 15s and UI shows `stopped`.
- **AE5.** Covers R14 — **When** team launches with 2+ agents on the same path without worktree flag, **Then** UI shows a non-blocking Git conflict warning.

---

## Success Criteria

- Operator can replace a typical **two-script parallel Claude** session with one UI session for the same project.
- Task state survives orchestrator restart; running agents do not survive restart (documented limitation).
- No requirement for Agent Teams AI feature parity in V1.

---

## Scope Boundaries

### In scope (V1)

Everything in R1–R16 (R13 = dev profile; R15–R16 = Docker/VPS profile).

### Deferred for later

- Agent-to-agent **mailbox** and cross-team messaging
- **Task dependencies / blockers**
- **Code review** UI (diff accept/reject)
- **Full execution logs** and token analytics
- **Multi-provider** runtimes (Codex, OpenCode, HTTP LLM)
- **Desktop** shell (Electron/Tauri)
- **Git worktree** per teammate
- **YAML workflow** runner as primary UX (may remain experimental)
- Sync with native Claude Code `~/.claude/tasks` team files
- **TLS termination** (use reverse proxy on VPS; not built into V1)
- **Authentication / API tokens** (required before exposing VPS to the internet—see plan risks)
- **Multi-tenant** SaaS

### Outside this product's identity (V1)

- Multi-user hosted SaaS with billing and org isolation
- Built-in code editor
- Replacing Claude Code CLI itself

---

## Key Decisions (product)

| Decision | Rationale |
|----------|-----------|
| Kanban-first, not workflow-first | User journey matches Agent Teams observation model; YAML stays secondary |
| Orchestrator DB is source of truth | Avoids brittle file-watcher on `~/.claude` for V1 |
| Mixed tasks via protocol | Mailbox deferred; structured actions keep scope bounded |
| Web UI; dev on localhost, prod in Docker on VPS | Remote control without desktop app; local dev for speed |
| Claude Code only | Matches existing repo direction and current workaround |

---

## Dependencies and Assumptions

- **Assumption:** Claude Code CLI is installed and on `PATH` (`claude --version` ≥ 2.1.32 recommended for future native-teams alignment).
- **Assumption:** Operator runs on a machine where spawning multiple `claude` processes is acceptable (RAM/CPU). On VPS, Claude Code CLI and credentials must be available **inside the container** (mounted config or env).
- **Assumption:** VPS exposure is **single-operator**; auth is added before public internet exposure (deferred in V1 implementation, not optional as a product concern).
- **Dependency:** Local SQLite (or equivalent) for persistence.
- **Blocking for planning (resolved in plan):** Long-running session model vs one-shot `claude chat --print` (plan chooses persistent sessions).

---

## Outstanding Questions

None blocking planning—all resolved in `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`.

## Related

- Plan: `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`
- Implemented architecture: `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
