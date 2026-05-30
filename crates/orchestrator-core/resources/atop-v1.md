# ATOP v1 — Agent Task Orchestration Protocol

Append **one JSON object per line** to `protocol.ndjson` (no array wrapper).

**Lead:** when the operator sends an `[orchestrator-objective]` message, append a `task.create` line first so the kanban board updates.

## Operations

### `task.create`
```json
{"op":"task.create","title":"Fix tests","description":"","status":"backlog","assignee":null}
```

### `task.update_status`
```json
{"op":"task.update_status","task_id":"UUID","status":"in_progress"}
```

### `task.assign`
```json
{"op":"task.assign","task_id":"UUID","assignee":"member-id-or-null"}
```

### `ping`
```json
{"op":"ping"}
```

Valid `status` values: `backlog`, `in_progress`, `review`, `done`.
