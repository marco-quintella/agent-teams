---
title: V1.3 browser QA — Svelte 5 effect loop freezes UI; launcher remount and headless browse
date: 2026-05-30
category: ui-bugs
module: claude-orchestrator
problem_type: ui_bug
component: development_workflow
symptoms:
  - "Console: effect_update_depth_exceeded after selecting or launching a team"
  - "Settings tab does not switch view; Board navigation feels frozen"
  - "Create & launch stuck on Launching… while API shows team running"
  - "After Settings → Board, create-new form shows despite a selected team"
  - "POST /api/setup/browse-directory returns 500 in headless/automation (rfd panic)"
root_cause: logic_error
resolution_type: code_fix
severity: critical
tags:
  - svelte
  - svelte-5
  - kanban
  - effect
  - untrack
  - team-launcher
  - rfd
  - browser-qa
  - orchestrator
related_components:
  - web
  - orchestrator-server
---

# V1.3 browser QA — Svelte 5 effect loop freezes UI; launcher remount and headless browse

## Problem

During V1.3 browser QA (saved teams, hybrid path browse, Settings default model), selecting or launching a team triggered a **Svelte 5 infinite effect loop** in the kanban board. The UI stopped processing navigation (Settings appeared clicked but the board stayed visible), launch buttons could remain on “Launching…”, and follow-up QA was unreliable until the reactive graph was fixed. Two smaller issues appeared in the same pass: **TeamLauncher** reset to “create new” when returning from Settings, and **`rfd`** panicked when Browse was invoked without a GUI (headless CI / browser automation).

## Symptoms

- Browser console: `Uncaught Error: https://svelte.dev/e/effect_update_depth_exceeded`
- **Settings** does not replace the board view when a team is already loaded (clicks seem ignored)
- **Create & launch** may leave `launchBusy` true in the UI while `GET /api/teams` shows `running`
- After **Settings → Board**, the sidebar shows the create-new form even though the kanban and agents reflect the selected team
- `curl -X POST /api/setup/browse-directory` in a non-windowed environment: **500** and server log panic (`NonWindowed environment…`) instead of a controlled error

## What Didn't Work

- Removing a `$derived(selectedSummary)` in `TeamLauncher.svelte` — reduced noise but did not stop the loop; the kanban `$effect` was the real source
- Fixing only the Browse handler’s `Option` nesting without `catch_unwind` — compile errors and still panicked in headless environments
- Assuming Settings routing was broken in `App.svelte` — `{#if view === 'settings'}` was correct; the effect loop prevented Svelte from applying view updates

## Solution

### 1. Kanban — sync store → columns inside `untrack`

`KanbanBoard.svelte` had a `$effect` that read `$tasks` and then **mutated** `$state` `columnItems` (and bumped `boardKey`). Svelte 5 tracked those writes as effect dependencies, re-ran the effect, and exceeded the update depth limit.

```svelte
import { untrack } from 'svelte';

$effect(() => {
  const list = $tasks;
  if (dragging) return;
  const next = columnsFromTasks(list);
  untrack(() => {
    if (columnsSignature(columnItems) === columnsSignature(next)) return;
    columnItems = next;
    boardKey += 1;
  });
});
```

Use a **column signature** (`id:status` per column) so WebSocket-driven `loadTeam` refreshes that do not change task placement do not remount dnd zones unnecessarily.

### 2. TeamLauncher — restore mode when remounted

`TeamLauncher` lives under `{#else}<main>` in `App.svelte`, so it **unmounts** when switching to Settings. On remount, `mode` defaulted to `create-new` while `teamId` remained in the global store.

```typescript
onMount(async () => {
  if (get(teamId)) {
    mode = 'selected';
  }
  // ...
});
```

Panel visibility:

```svelte
{#if $teamId && mode !== 'create-new'}
  <!-- selected panel -->
{:else}
  <!-- create panel -->
{/if}
```

### 3. Browse directory — `catch_unwind` + 503

Wrap `rfd::FileDialog::pick_folder()` in `std::panic::catch_unwind` inside `spawn_blocking`. Map outcomes explicitly:

| `catch_unwind` result | HTTP |
|----------------------|------|
| `Ok(Some(path))` | 200 + `{ path }` |
| `Ok(None)` (cancelled) | 400 `cancelled` |
| `Err(_)` (panic / no GUI) | 503 `native folder dialog unavailable…` |

```rust
let pick_result = tokio::task::spawn_blocking(move || {
    std::panic::catch_unwind(|| {
        let mut dialog = rfd::FileDialog::new();
        if let Some(dir) = initial {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_folder()
    })
})
.await?;
```

The UI shows a friendly message when Browse returns 503 (type path manually).

## Why This Works

- **`untrack`** prevents writes to `columnItems` / `boardKey` from registering as dependencies of the effect that syncs from `$tasks`, breaking the feedback loop.
- **Signature guard** avoids redundant dnd remounts when the store refreshes with the same logical columns.
- **onMount + store `teamId`** reconciles ephemeral component state with durable selection after view toggles.
- **`catch_unwind`** turns `rfd`’s environment panic into an API error the client can handle; automation and headless servers no longer crash the Axum worker.

## Prevention

- In Svelte 5, treat **`$effect` + `$state` writes** as suspect: either sync via `$derived` for read-only projections, or wrap mutations in `untrack()`.
- When a child component holds **local UI mode** but **global stores** hold selection, remount paths (tab switches, `{#if}` view toggles) must **rehydrate** local state in `onMount` or lift mode to the parent.
- Native dialogs (`rfd`, file pickers) in server handlers: always **`spawn_blocking` + `catch_unwind`**, return **503** when GUI is unavailable.
- Browser QA after UI changes: select team → open kanban → **Settings ↔ Board** → create & launch; watch console for `effect_update_depth_exceeded`.
- Do not confuse this with API hangs from PTY on the Tokio runtime — see `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md` (different root cause).

## Related Issues

- Architecture (Svelte UI, kanban, launcher): `docs/solutions/architecture-patterns/claude-orchestrator-v1-stack.md`
- PTY / launch “busy” hangs (V1.1): `docs/solutions/performance-issues/orchestrator-pty-blocking-tokio-runtime.md`
- V1.2 operator workspace (Settings, single-serve): `docs/solutions/developer-experience/orchestrator-v1.2-self-contained-localhost.md`
- V1.3 plan: `docs/plans/2026-05-30-004-feat-agent-orchestrator-v1.3-plan.md`
- V1.3 requirements: `docs/brainstorms/2026-05-30-agent-orchestrator-v1.3-requirements.md`
