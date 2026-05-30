---
title: "Agent Orchestrator V1.1"
status: active
date: 2026-05-30
type: requirements
origin: docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md
origin_conversation: ce-brainstorm 2026-05-30 (V1.1 hardening)
---

## Summary

V1.1 **closes the V1 product promise on localhost**: launch a **full team** (lead + N workers) with **real persistent Claude Code sessions** (no `claude --version` placeholder), deliver **team objectives to the lead inside the live session**, and prove success when the **lead creates at least one kanban task via the agent protocol** that reflects the objective within a few minutes.

V1 scaffold (API, WebSocket, SQLite, UI) stays; V1.1 replaces the demo spawn path and makes **R3, R8, and R11** work end-to-end in development.

---

## Problem Frame

The V1 implementation compiles and exposes kanban, persistence, and supervisor plumbing, but teammates still launch as a **placeholder CLI invocation**, and lead objectives are **file-only** without reliable delivery into the session. Operators cannot yet replace parallel Claude scripts with a trustworthy UI loop.

V1.1 is a **hardening release**, not feature expansion toward Agent Teams AI parity.

---

## Actors

Same as V1 (`docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md`):

- **A1 — Operator**
- **A2 — Lead agent**
- **A3 — Worker agent**
- **A4 — Orchestrator**

---

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| **Approach A — interactive session per member** | Each teammate runs a **persistent** Claude Code session under supervisor control; objectives to the lead go through the **live session input channel** (primary), not only a sidecar file. |
| **Inbound file as backup/audit trail** | Appending to the member inbound sidecar may continue for durability, but **must not be the only delivery path** for operator objectives to the lead. |
| **Proof on lead task creation only** | V1.1 is done when the **lead** creates a protocol-backed task coherent with the objective; workers must **launch** with real sessions but need not move cards or complete work. |
| **Local dev profile first** | Primary validation is **localhost** (V1 R13). Auth and mandatory Docker smoke are explicitly **not** V1.1 gates. |
| **V1 deferred list unchanged** | Mailbox, blockers, code review, multi-provider, worktrees, full logs remain out of scope. |

---

## Key Flows

- **F1 — Launch real team:** Operator launches lead + all configured workers; each member enters `starting` → `running` with a **long-running** Claude session (not a one-shot version check).
- **F2 — Send objective to lead:** Operator sends a text objective; orchestrator delivers it into the **lead's live session**; operator sees activity in status/snippet within a reasonable window.
- **F3 — Lead creates task:** Lead appends a valid **agent protocol** create-task action; orchestrator ingests it; a new card appears on the kanban in **Backlog** (or chosen column per protocol) with `created_by=agent`; UI updates without refresh.
- **F4 — Observe full team:** Operator sees all members' status and snippets while only requiring lead task creation for the release proof.
- **F5 — Stop team:** Unchanged from V1 — all sessions terminate and UI shows `stopped`.

---

## Requirements

**Session and launch**

- **R1.** On launch, **every** team member (lead and all workers) starts a **persistent** Claude Code CLI session scoped to the project root with role context—not `claude --version` or other placeholder commands.
- **R2.** If Claude CLI is missing, launch **fails clearly** for the team (no silent mock in production path unless operator explicitly enables a test/mock profile).
- **R3.** Each session receives bootstrap context sufficient to use the agent protocol (role, team provisioning summary, protocol capability summary, current board snapshot).

**Lead objective delivery**

- **R4.** Operator can send a **team objective message** to the lead from the UI (V1 R11).
- **R5.** The orchestrator delivers that message into the **lead's live session input** as the primary path.
- **R6.** Delivery is **observable**: within 60s the lead's last-output snippet or status reflects receipt (e.g., visible prompt echo, activity, or explicit acknowledgment in output)—not only a successful HTTP response.

**Agent protocol and kanban**

- **R7.** The orchestrator **continuously ingests** agent protocol actions from each live member's protocol stream while the team is running (V1 R8).
- **R8.** When the lead emits a valid **create task** action, a task appears on the board with title/description consistent with the action and `created_by=agent`.
- **R9.** Task and agent updates continue to push to connected UI clients in real time (V1 R9).

**Team composition and operator proof**

- **R10.** Launch supports **lead + N workers** as configured in the team launcher (full team, not lead-only).
- **R11.** V1.1 release proof: after sending an objective, **at least one new task** created by the **lead** via protocol appears within **5 minutes** under normal conditions (installed Claude, authenticated CLI, modest project).

**Regression / carry-forward**

- **R12.** V1 requirements **R1–R7, R9–R10, R12–R14** remain satisfied on localhost dev profile (human kanban CRUD, stop/restart, persistence, Git warning).
- **R13.** Documented limitation unchanged: orchestrator restart does not resurrect running agents.

**Explicitly not required in V1.1**

- **R14.** Workers are **not** required to create tasks, change status, or accept assignments for V1.1 acceptance.
- **R15.** Auth/API tokens are **not** required in V1.1 (still mandatory before public VPS exposure—see V1 scope).
- **R16.** Formal Docker-compose smoke test is **not** a V1.1 acceptance gate (may be manual spot-check only).

---

## Acceptance Examples

- **AE1.** Covers R1, R3, R10, F1 — **Given** valid project path and `claude` on PATH, **When** operator launches a team with lead and two workers, **Then** three members show `running` or `starting` within 60s and snippets show session activity (not merely a version string).
- **AE2.** Covers R4–R6, F2 — **Given** a running lead, **When** operator sends objective "Add a CONTRIBUTING.md with install steps", **Then** within 60s the lead snippet or status shows the objective was received.
- **AE3.** Covers R7–R8, R11, F3 — **Given** AE2, **When** the lead emits a valid protocol create-task for work related to the objective, **Then** a new Backlog card appears with `created_by=agent` on all connected UI clients without refresh, within 5 minutes.
- **AE4.** Covers R12, F5 — **When** operator stops the team, **Then** all sessions terminate within 15s and UI shows `stopped`.
- **AE5.** Covers R14 (negative) — **Given** workers never emit protocol actions, **When** AE3 passes for the lead, **Then** V1.1 is still considered satisfied.

---

## Success Criteria

- Operator can run **one localhost UI session** instead of parallel scripts for the same project: launch team → send objective → see **lead-created task** on the kanban.
- README and operator docs no longer describe placeholder spawn as the normal behavior.
- Clear documented gap to **V1.2**: auth for VPS, optional Docker verification, worker-driven task flows.

---

## Scope Boundaries

### In scope (V1.1)

Everything in R1–R13 and AE1–AE5.

### Deferred for later (unchanged from V1; not V1.1)

- Agent-to-agent **mailbox** and cross-team messaging
- **Task dependencies / blockers**
- **Code review** UI
- **Full execution logs** and token analytics
- **Multi-provider** runtimes
- **Git worktree** per teammate
- **Authentication / API tokens** as implemented product feature (V1.2+)
- Mandatory **Docker/VPS** acceptance automation
- Agent Teams AI parity

### Outside this product's identity

Unchanged from V1 (no SaaS billing, no built-in editor, no replacing Claude Code CLI).

---

## Dependencies and Assumptions

- **Dependency:** V1 scaffold in repo (`orchestrator-core`, `orchestrator-server`, `web/`).
- **Dependency:** ATOP v1 ingestor and protocol spec (`crates/orchestrator-core/resources/atop-v1.md`).
- **Assumption:** Operator validates on **Windows or Linux dev machine** with Claude Code CLI installed and authenticated.
- **Assumption:** Exact Claude CLI flags for persistent interactive sessions are confirmed during planning/implementation against installed `claude --help` (V1 open question—**resolved in V1.1** toward persistent sessions, not one-shot print).
- **Assumption:** Lead role prompt (provisioning + role fragment) instructs the lead to use protocol for task creation when given an objective.
- **Risk:** CLI behavior changes between Claude Code versions may require prompt or invocation adjustments—document minimum supported version after verification.

---

## Outstanding Questions

None blocking planning—approach **A** confirmed in brainstorm dialogue.

---

## Related

- Parent: `docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md`
- V1 plan: `docs/plans/2026-05-30-001-feat-agent-orchestrator-v1-plan.md`
- Architecture (current): `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
- Troubleshooting (API hang / PTY on Tokio): `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`
