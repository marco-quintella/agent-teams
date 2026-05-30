# ATOP v1 — Agent Task Orchestration Protocol

Append **one JSON object per line** to `protocol.ndjson` (no array wrapper).

**Lead:** when the operator asks for board or task changes, use ATOP (`task.create`, etc.). For other objectives, respond in session; use ATOP only when it matches what was asked.

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
