---
title: "Agent Orchestrator V1.3"
status: active
date: 2026-05-30
type: requirements
origin: ce-brainstorm 2026-05-30 (V1.3 operator workspace — folder picker, team history, model in settings)
parent: docs/brainstorms/2026-05-30-agent-orchestrator-v1.2-requirements.md
---

## Summary

V1.3 improves **operator repeatability** on localhost: **browse** for a project directory, **select and relaunch** teams already stored in SQLite, and set a **default Claude model** in Settings for every interactive member spawn—without new browser `localStorage` keys.

Builds on V1.2 (credentials, doctor, single-process serve, PTY sessions, lead autonomy). Scope is launcher and Settings UX, not orchestrator HTTP auth or team editing.

---

## Problem Frame

After V1.2, the operator can configure Claude credentials and launch teams, but daily use still forces repetitive setup: typing absolute project paths by hand, recreating teams from scratch on every session, and running all members on whatever default model the CLI picks because Settings has no model control.

The orchestrator already persists projects, teams, members, and tasks in SQLite, yet the UI only exposes **create** flows—not **select and relaunch** existing teams. The launcher binds `projectPath` as a plain text field with no folder picker.

V1.3 closes that gap: pick a repo safely, pick a saved team, launch with a configured model—server/API is the source of truth for team history.

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| **Team history lives on the server** | Operator selects from teams already stored in SQLite; no named preset library or launcher-only defaults in scope. |
| **Team list actions: launch only** | View teams, select one, launch (or stop)—no edit, delete, or duplicate in this release. |
| **Hybrid project path entry** | Keep manual path input; add **Browse…** that opens a native folder dialog when the backend can drive the OS (localhost operator tool). |
| **No additional browser persistence** | Do not add localStorage for paths, UI tabs, or form defaults; server/API is the source of truth. Existing `localStorage` team resume may be removed or replaced by explicit server selection during implementation. |
| **Settings: global Claude model only** | One default model applied to interactive `claude` spawns for all members via CLI `--model`; not per-member overrides, not multi-provider agent types in this release. |
| **Carry V1.2 credential scope** | Settings continues to own Claude Code credentials (CLI login vs API key, optional base URL); model is an additional field in the same surface. |

## Requirements

**Project path**

- **R1.** When creating a **new** team, the launcher exposes **Project path** with a text field **and** a **Browse…** control.
- **R2.** **Browse…** opens a **native directory picker** on the operator machine when supported (hybrid path); the chosen directory populates the path field with an absolute path validated by the orchestrator (existing rules: non-empty, no `..`).
- **R3.** If native browse is unavailable (unsupported OS/API failure), the operator can still set the path by typing; the UI shows a clear message that browse failed and manual entry remains available.
- **R4.** Creating or launching against a project path continues to use the existing project record model (path stored server-side when a team is created or selected).

**Team history**

- **R5.** The operator sees a **list of existing teams** from server persistence (name, project path or label, created date, running/stopped indication when inferable).
- **R6.** Selecting a team loads its board context (tasks, members, agent runs) without requiring the operator to re-enter team configuration in the launcher form. The stored project path is shown **read-only**; **Browse…** is disabled in this state.
- **R7.** **Launch** on a selected team starts supervised sessions for that team’s members (same semantics as today’s launch after create).
- **R8.** **Stop** remains available for the **selected** team as today.
- **R9.** Creating a **new** team (new name, path, members, provisioning prompt) remains supported alongside history—not replaced by history-only flow.

**Settings — model**

- **R10.** Settings includes a **default model** for Claude Code sessions spawned by the orchestrator.
- **R11.** The chosen model is **persisted server-side** with other Claude settings and applied on every member spawn (interactive PTY), not only on one-shot CLI calls.
- **R12.** Doctor or Settings copy indicates when no model is configured and which default the product will use (implementation may document CLI fallback); launch must not fail silently on a wrong model—surface CLI errors in agent status as today.

**Persistence and state**

- **R13.** This release adds **no new** browser `localStorage` keys for paths, UI tabs, form defaults, or team selection. Existing active-team resume (if any) may be removed or replaced per implementation (see Outstanding Questions); transient UI-only state is allowed only when unavoidable.
- **R14.** Reloading the UI does not force the operator to recreate a team if that team still exists server-side; they can select it from the list again.

**Regression**

- **R15.** V1.2 flows still work: doctor, credentials, single-process localhost serve, lead messaging, human kanban, ATOP ingest, stop/relaunch without zombie sessions.

## Success Criteria

- Operator picks a repo folder via **Browse…** (or typed path), selects a **previously created team** from the list, sets a **model in Settings**, launches, and sees agents run under that model without retyping team setup.
- Fresh operator can still **create** a new team end-to-end (path + members + launch).
- No new dependency on browser-only directory APIs as the **only** path to a valid absolute path on desktop workflows.

## Scope Boundaries

### In scope

- R1–R15 and acceptance examples below.

### Out of scope

- Edit, delete, or duplicate team from the history list.
- Named team templates / preset library separate from persisted teams.
- Per-member model overrides in the launcher.
- Agent type selection (OpenRouter vs Claude Code) beyond existing credential mode + base URL.
- Project browser tree inside the web UI without native dialog.
- Orchestrator HTTP/API authentication.
- Git worktrees, mailbox, multi-provider runtimes.

## Acceptance Examples

- **AE1.** **Given** teams already exist in the database, **When** the operator opens the app, **Then** they see the team list and can select one without using the create form.
- **AE2.** **Given** a selected team and configured credentials + model, **When** the operator launches, **Then** all members spawn and doctor/credentials gates still apply.
- **AE3.** **Given** the native folder picker is available, **When** the operator clicks **Browse…** and chooses a repo directory, **Then** the path field shows that absolute path and a new team can be created against it.
- **AE4.** **Given** model set to a non-default id in Settings, **When** a team launches, **Then** spawned sessions use that model (verifiable via CLI process args or documented inspection path on maintainer machine).
- **AE5.** **Given** browse is unsupported or denied, **When** the operator types a valid path, **Then** create/launch still succeeds (AE3 optional).

## Dependencies and Assumptions

- **Dependency:** V1.2 shipped (SQLite teams/projects, Settings, supervisor PTY spawn, single local serve).
- **Assumption:** List/read APIs for teams (and linked project paths) will be added—data already exists in SQLite from V1 create flows; the UI today does not list them.
- **Assumption:** Native directory dialog is acceptable for a localhost-only operator tool (Windows and Linux maintainer targets per V1.2).
- **Unverified:** Exact Claude CLI flag for model on **interactive** sessions matches one-shot `--model` behavior—confirm during planning against installed CLI version.

## Outstanding Questions

### Resolve Before Planning

- *(none — model picker UX skipped in brainstorm; see deferred default below)*

### Deferred to Planning

- **[Affects R10][User decision]** Model selection control: curated dropdown, free text, or curated + custom—default assumption if unanswered: **curated list + optional custom id**.
- **[Affects R2][Technical]** Native dialog crate/strategy per OS and how the API returns the path to the web UI (sync HTTP vs job poll).
- **[Affects R13][Technical]** Whether to remove `localStorage` active-team resume entirely in favor of server-side “last selected team id” or no auto-selection (see R13).
- **[Affects R11][Needs research]** Minimum Claude Code CLI version and flags for `--model` on persistent sessions.

## Related

- Parent: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.2-requirements.md`
- Plan: `docs/plans/2026-05-30-004-feat-agent-orchestrator-v1.3-plan.md`
- Post-implementation (browser QA fixes): `docs/solutions/ui-bugs/orchestrator-v1.3-svelte-effect-loop-and-launcher-remount.md`

## Next Steps

-> `/ce-work` to implement V1.3 (see plan)
