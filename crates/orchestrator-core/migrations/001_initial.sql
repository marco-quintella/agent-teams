CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    root_path TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE teams (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,
    provisioning_prompt TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE TABLE team_members (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id),
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    role_prompt TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL,
    assignee_member_id TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE task_events (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    actor TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE TABLE agent_runs (
    id TEXT PRIMARY KEY NOT NULL,
    team_member_id TEXT NOT NULL REFERENCES team_members(id),
    status TEXT NOT NULL,
    last_output_snippet TEXT NOT NULL DEFAULT '',
    pid INTEGER,
    started_at TEXT,
    stopped_at TEXT,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_tasks_team_status ON tasks(team_id, status);
CREATE INDEX idx_team_members_team ON team_members(team_id);
CREATE INDEX idx_agent_runs_member ON agent_runs(team_member_id);
