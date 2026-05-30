---
title: "Agent Orchestrator V1.2"
status: active
date: 2026-05-30
type: requirements
origin: docs/brainstorms/2026-05-30-agent-orchestrator-v1.1-requirements.md
origin_conversation: ce-brainstorm 2026-05-30 (V1.2 self-contained operator + lead autonomy)
---

## Summary

V1.2 makes the orchestrator **self-sufficient for the operator on localhost**: one executable serves API and UI, **Settings** configures Claude Code credentials (CLI login or API key, including third-party keys such as OpenRouter for testing), guided CLI install and **doctor** in-product, and **reliable spawn and session lifecycle** for every teammate—without requiring the Claude Code desktop app to be open.

Objectives to the **lead** work like a **conversation**: the operator asks; the lead, as team leader, **decides what to do** (including whether and how to use ATOP). The orchestrator **must not** deterministically create kanban tasks or append protocol lines on the agent's behalf. V1.1 hardening issues documented in-repo are **fixed or materially reduced** as part of this release.

---

## Problem Frame

V1.1 replaced placeholder spawn and wired lead objectives through PTY, but operators still depend on **manual CLI setup**, **credentials outside the product**, and often **two processes** (API + Vite) for local use. Documented failures include API stalls from blocking PTY work, lead sessions dying with closed pipes, and fragile proof when objectives are vague or the orchestrator is expected to "force" outcomes.

V1.2 is an **operator-autonomy and reliability** release: the tool owns Claude CLI lifecycle and configuration, while preserving **agent judgment** at the lead—especially for task creation on the kanban.

---

## Actors

Same as V1 (`docs/brainstorms/2026-05-30-agent-orchestrator-v1-requirements.md`):

- **A1 — Operator**
- **A2 — Lead agent** (team leader; autonomous decisions within role)
- **A3 — Worker agent**
- **A4 — Orchestrator** (spawn, delivery, ingest—**not** a substitute decision-maker for the lead)

---

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| **Lead autonomy over outcomes** | The lead interprets operator messages and chooses actions (create task, clarify, delegate in future versions, etc.). Success is **appropriate response to a clear ask**, not orchestrator-side protocol synthesis. |
| **No deterministic ATOP relay** | The orchestrator **does not** run shadow `claude -p` turns, append `protocol.ndjson`, or otherwise bypass the lead to "guarantee" a card on the board. |
| **Conversational objectives** | Operator ↔ lead messages are natural-language requests; validation uses **explicit** asks (e.g. "create a backlog task for X"), not trivial pings that invite random task creation. |
| **Human kanban unchanged** | Operator may always create or edit tasks on the board directly; that path is independent of lead chat. |
| **Settings scope = Claude Code only** | One settings surface for CLI path, login, and API key (incl. compatible third-party keys). **Orchestrator HTTP/API auth** for exposing the tool on a network is **deferred**. |
| **Credential choice** | Operator picks **CLI login** (guided) **or** **API key** stored for child spawns; both are first-class, not a single mandated path. |
| **Single local executable** | On localhost, one `serve` (or equivalent) delivers API **and** UI—no separate Vite terminal required for the default dev loop. |
| **Docker = production path** | Production deployment remains Docker-based; **V1.2 acceptance is localhost only** (Windows/Linux dev machines). |
| **V1.1 session model retained** | Persistent interactive `claude` per member under supervisor PTY; V1.2 hardens and operationalizes it rather than replacing with one-shot-only agents. |
| **V1 deferred list largely unchanged** | Mailbox, blockers, code review UI, multi-provider agent runtimes, worktrees, orchestrator API tokens remain out of scope unless noted below. |

---

## Key Flows

- **F1 — First run / doctor:** Operator starts the app; **doctor** reports orchestrator version, CLI presence, version, and auth readiness; missing CLI offers **install via official script** (confirmed subprocess), not silent bundling.
- **F2 — Configure Claude:** Operator opens **Settings**, chooses login-via-CLI or API key, saves; doctor reflects ready state before launch.
- **F3 — Launch team:** Operator launches lead + workers; orchestrator spawns and supervises all `claude` sessions; no separate Claude Code app required.
- **F4 — Talk to the lead:** Operator sends a clear objective (e.g. create a named backlog task); orchestrator delivers into the **live lead session**; lead activity appears in snippet/status; lead may create ATOP tasks **if it judges that appropriate to the request**.
- **F5 — Human kanban:** Operator creates or moves tasks manually at any time without messaging the lead.
- **F6 — Ingest agent tasks:** When the lead (or any member) appends valid ATOP lines, cards update in real time as today.
- **F7 — Stop team:** All sessions terminate; UI shows stopped; no orphaned processes blocking relaunch.

---

## Requirements

**Operator experience (local)**

- **R1.** A **single local command** starts API and web UI together on localhost (default dev loop documented as one executable/process model).
- **R2.** **Settings** (dedicated UI) configures Claude Code only: credential mode (CLI login vs API key), optional provider-compatible key (e.g. OpenRouter for testing), and displays doctor summary.
- **R3.** **Doctor** is available from CLI and surfaced in UI; reports CLI found/missing, version, and whether configured credentials appear usable before launch.
- **R4.** When CLI is missing, operator can trigger **guided install** (official install script via subprocess with confirmation)—not a silent auto-install without consent.

**Sessions and spawn**

- **R5.** On launch, every member (lead + workers) runs a **persistent** supervised `claude` session scoped to project root and role context (carry V1.1 R1–R3).
- **R6.** Launch **fails clearly** if CLI is missing or credentials are not configured (no silent mock in production path).
- **R7.** Lead objectives are delivered through the **live PTY channel** (primary), with inbound sidecar as audit only (carry V1.1 R4–R6 intent).
- **R8.** API remains responsive during launch and message delivery (no blocking PTY on async executor—carry fix from `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`).
- **R9.** **Session health** is observable: if the lead child exits, operator sees failed/stopped state and a clear error (e.g. pipe closed)—not a hung UI with a false success.
- **R10.** Relaunch stops prior team sessions before spawning replacements (no zombie `claude` blocking pipes).

**Lead behavior and protocol**

- **R11.** The orchestrator **never** appends ATOP lines or creates agent-attributed tasks **on behalf of** the lead to satisfy acceptance.
- **R12.** Operator messages to the lead are **conversational**; the lead decides actions consistent with role and the request.
- **R13.** When the operator asks the lead to create a kanban item (explicit validation scenario), the lead **may** emit `task.create` via ATOP; wording and decomposition are the lead's choice.
- **R14.** ATOP ingest and WebSocket kanban updates remain continuous while sessions run (carry V1.1 R7–R9).
- **R15.** Human task CRUD and drag-and-drop on the board work without lead involvement (carry V1.1 R12 regression).

**V1.1 defect remediation**

- **R16.** Documented V1.1 issues are addressed: PTY/Tokio blocking, conflated UI busy flags, stale team state on failed resume, Windows operational notes (exe lock on rebuild), and Svelte-bound fields that block Launch until blur—verified on maintainer localhost checklist.

**Regression / carry-forward**

- **R17.** Full team launch (lead + N workers) remains supported (carry V1.1 R10).
- **R18.** Orchestrator restart does not resurrect agents (documented limitation unchanged).

**Explicitly not required in V1.2**

- **R19.** Workers need not create tasks or execute work for release acceptance (carry V1.1 R14).
- **R20.** Orchestrator **HTTP/API authentication** (LAN/VPS protection) is not in scope—only Claude credential configuration.
- **R21.** Mandatory Docker-compose acceptance gate is not required (Docker remains the documented **production** path).
- **R22.** Forcing a kanban card within a rigid timeout when the operator did not ask for task creation (e.g. sending "hello") is **not** a valid acceptance scenario.

---

## Acceptance Examples

- **AE1.** Covers R1, R3, F1 — **Given** a fresh machine with Rust built, **When** operator runs the single local start command and opens the UI, **Then** doctor is reachable and the kanban loads without a separate Vite terminal.
- **AE2.** Covers R2, R4, F2 — **Given** CLI missing, **When** operator uses guided install then configures API key (or CLI login) in Settings, **Then** doctor shows CLI present and credentials ready.
- **AE3.** Covers R5–R7, R9–R10, F3–F4 — **Given** configured credentials, **When** operator launches lead + one worker and sends an explicit objective such as *"Create a backlog task titled 'CONTRIBUTING setup' for adding install steps to CONTRIBUTING.md"*, **Then** within a reasonable window (~15 min under normal CLI/auth) the lead snippet shows processing and either (a) a matching `created_by=agent` card appears, or (b) the lead's visible output explains why it is not creating a task (operator can judge misconfiguration vs refusal).
- **AE4.** Covers R11–R13 (negative control) — **When** operator sends a trivial message with no task ask (e.g. "hello"), **Then** V1.2 is **not** failed solely because no agent task appeared; passing AE3 is the protocol proof path.
- **AE5.** Covers R15, F5 — **When** operator creates a task manually on the board, **Then** it appears without lead action.
- **AE6.** Covers R8, R16, F7 — **When** operator stops the team or relaunches, **Then** API stays responsive, sessions end within ~15s, and no pipe-closed success on a dead lead.
- **AE7.** Covers R20 (negative) — **Given** localhost bind only, **When** no orchestrator API token is configured, **Then** V1.2 acceptance still passes (orchestrator auth is a later release).

---

## Success Criteria

- Operator can install/configure Claude **inside the product**, start **one local binary**, launch a team, and **converse with the lead** using explicit work requests—without opening Claude Code separately.
- Lead-driven kanban updates happen when the lead chooses ATOP actions aligned with the request; the product does not cheat with orchestrator-written protocol.
- README and operator docs describe the single-command local loop and Settings; V1.1 troubleshooting items are resolved or downgraded to edge cases with clear UX.
- Docker path documented for production; localhost checklist signed off before tag.

---

## Scope Boundaries

### In scope (V1.2)

Everything in R1–R18 and AE1–AE7.

### Deferred for later

- Orchestrator **HTTP/API token** auth and TLS guidance for public VPS
- Mandatory automated Docker smoke as release gate
- Worker-driven task flows as acceptance requirement
- Mailbox, blockers, code review UI, multi-provider agent runtimes, git worktrees per member
- YAML workflow execution engine
- Native `~/.claude/tasks` sync

### Outside this product's identity

Unchanged from V1: not a SaaS billing product, not a code editor replacement, not a full fork of Claude Code—**we orchestrate the CLI the operator configures**.

---

## Dependencies and Assumptions

- **Dependency:** V1.1 codebase (supervisor PTY, ATOP ingestor, Svelte UI, SQLite).
- **Dependency:** ATOP v1 spec (`crates/orchestrator-core/resources/atop-v1.md`).
- **Assumption:** Validation on **Windows or Linux** localhost with network for CLI install/login as needed.
- **Assumption:** Third-party API keys (e.g. OpenRouter) work when passed through mechanisms supported by the installed Claude Code CLI—**verify during planning**; document supported env/flags after first successful test.
- **Assumption:** Lead `role.md` and bootstrap continue to describe ATOP capability; improving prompts is in scope, but **outcome forcing is not**.
- **Risk:** CLI flag or auth behavior drift across Claude Code versions—doctor reports version; pin minimum after verification.

---

## Outstanding Questions

- Exact storage for API keys (SQLite vs OS keychain vs encrypted file)—planning decision; must not log secrets.
- Whether guided `claude login` runs in embedded terminal, subprocess with URL copy, or external browser handoff—planning UX detail.

---

## Related

- Parent: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.1-requirements.md`
- V1.1 plan: `docs/plans/2026-05-30-002-feat-agent-orchestrator-v1.1-plan.md`
- V1.1 troubleshooting: `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`
- Architecture: `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
