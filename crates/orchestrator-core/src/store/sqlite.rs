use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;

use super::Store;
use crate::claude_settings::{settings_row_id, CredentialMode, ClaudeSettings};
use crate::domain::{
    AgentRun, AgentRunStatus, MemberRole, Project, Task, TaskActor, TaskEvent, TaskStatus, Team,
    TeamMember, new_id,
};

/// SQLite-backed store.
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    fn parse_dt(s: &str) -> anyhow::Result<DateTime<Utc>> {
        Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc))
    }
}

#[async_trait]
impl Store for SqliteStore {
    async fn create_project(&self, root_path: &str) -> anyhow::Result<Project> {
        let now = Utc::now();
        let project = Project {
            id: new_id(),
            root_path: root_path.to_string(),
            created_at: now,
        };
        sqlx::query(
            "INSERT INTO projects (id, root_path, created_at) VALUES (?, ?, ?)",
        )
        .bind(&project.id)
        .bind(&project.root_path)
        .bind(project.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(project)
    }

    async fn get_project(&self, id: &str) -> anyhow::Result<Option<Project>> {
        let row = sqlx::query("SELECT id, root_path, created_at FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| {
            Project {
                id: r.get("id"),
                root_path: r.get("root_path"),
                created_at: SqliteStore::parse_dt(r.get::<String, _>("created_at").as_str())
                    .unwrap_or_else(|_| Utc::now()),
            }
        }))
    }

    async fn create_team(
        &self,
        project_id: &str,
        name: &str,
        provisioning_prompt: &str,
    ) -> anyhow::Result<Team> {
        let now = Utc::now();
        let team = Team {
            id: new_id(),
            project_id: project_id.to_string(),
            name: name.to_string(),
            provisioning_prompt: provisioning_prompt.to_string(),
            created_at: now,
        };
        sqlx::query(
            "INSERT INTO teams (id, project_id, name, provisioning_prompt, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&team.id)
        .bind(&team.project_id)
        .bind(&team.name)
        .bind(&team.provisioning_prompt)
        .bind(team.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(team)
    }

    async fn add_team_member(
        &self,
        team_id: &str,
        name: &str,
        role: MemberRole,
        role_prompt: &str,
    ) -> anyhow::Result<TeamMember> {
        let now = Utc::now();
        let member = TeamMember {
            id: new_id(),
            team_id: team_id.to_string(),
            name: name.to_string(),
            role,
            role_prompt: role_prompt.to_string(),
            created_at: now,
        };
        sqlx::query(
            "INSERT INTO team_members (id, team_id, name, role, role_prompt, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&member.id)
        .bind(&member.team_id)
        .bind(&member.name)
        .bind(member.role.as_str())
        .bind(&member.role_prompt)
        .bind(member.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(member)
    }

    async fn list_team_members(&self, team_id: &str) -> anyhow::Result<Vec<TeamMember>> {
        let rows = sqlx::query(
            "SELECT id, team_id, name, role, role_prompt, created_at FROM team_members WHERE team_id = ? ORDER BY created_at",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                let role_str: String = r.get("role");
                Ok(TeamMember {
                    id: r.get("id"),
                    team_id: r.get("team_id"),
                    name: r.get("name"),
                    role: MemberRole::parse(&role_str)
                        .ok_or_else(|| anyhow::anyhow!("invalid role: {role_str}"))?,
                    role_prompt: r.get("role_prompt"),
                    created_at: SqliteStore::parse_dt(r.get::<String, _>("created_at").as_str())?,
                })
            })
            .collect()
    }

    async fn get_team(&self, team_id: &str) -> anyhow::Result<Option<Team>> {
        let row = sqlx::query(
            "SELECT id, project_id, name, provisioning_prompt, created_at FROM teams WHERE id = ?",
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Team {
            id: r.get("id"),
            project_id: r.get("project_id"),
            name: r.get("name"),
            provisioning_prompt: r.get("provisioning_prompt"),
            created_at: Self::parse_dt(r.get::<String, _>("created_at").as_str())
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Task>> {
        let row = sqlx::query(
            "SELECT id, team_id, title, description, status, assignee_member_id, created_by, created_at, updated_at FROM tasks WHERE id = ?",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|r| Self::row_to_task(r)).transpose()
    }

    async fn create_task(
        &self,
        team_id: &str,
        title: &str,
        description: &str,
        status: TaskStatus,
        assignee_member_id: Option<&str>,
        created_by: TaskActor,
    ) -> anyhow::Result<Task> {
        let now = Utc::now();
        let task = Task {
            id: new_id(),
            team_id: team_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status,
            assignee_member_id: assignee_member_id.map(str::to_string),
            created_by,
            created_at: now,
            updated_at: now,
        };

        sqlx::query(
            "INSERT INTO tasks (id, team_id, title, description, status, assignee_member_id, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&task.id)
        .bind(&task.team_id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status.as_str())
        .bind(&task.assignee_member_id)
        .bind(task.created_by.as_str())
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.record_task_event(&task.id, created_by, "task.created", "{}")
            .await?;

        Ok(task)
    }

    async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        actor: TaskActor,
    ) -> anyhow::Result<Task> {
        let now = Utc::now();
        sqlx::query("UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(now.to_rfc3339())
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        let payload = serde_json::json!({ "status": status.as_str() }).to_string();
        self.record_task_event(task_id, actor, "task.status_changed", &payload)
            .await?;

        self.get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("task not found after update"))
    }

    async fn assign_task(
        &self,
        task_id: &str,
        assignee_member_id: Option<&str>,
        actor: TaskActor,
    ) -> anyhow::Result<Task> {
        let now = Utc::now();
        sqlx::query("UPDATE tasks SET assignee_member_id = ?, updated_at = ? WHERE id = ?")
            .bind(assignee_member_id)
            .bind(now.to_rfc3339())
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        let payload = serde_json::json!({ "assignee": assignee_member_id }).to_string();
        self.record_task_event(task_id, actor, "task.assigned", &payload)
            .await?;

        self.get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("task not found after assign"))
    }

    async fn list_tasks(&self, team_id: &str) -> anyhow::Result<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT id, team_id, title, description, status, assignee_member_id, created_by, created_at, updated_at
             FROM tasks WHERE team_id = ? ORDER BY updated_at DESC",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_task(r)).collect()
    }

    async fn record_task_event(
        &self,
        task_id: &str,
        actor: TaskActor,
        event_type: &str,
        payload_json: &str,
    ) -> anyhow::Result<TaskEvent> {
        let now = Utc::now();
        let event = TaskEvent {
            id: new_id(),
            task_id: task_id.to_string(),
            actor,
            event_type: event_type.to_string(),
            payload_json: payload_json.to_string(),
            created_at: now,
        };

        sqlx::query(
            "INSERT INTO task_events (id, task_id, actor, event_type, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.task_id)
        .bind(event.actor.as_str())
        .bind(&event.event_type)
        .bind(&event.payload_json)
        .bind(event.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(event)
    }

    async fn upsert_agent_run(&self, run: &AgentRun) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO agent_runs (id, team_member_id, status, last_output_snippet, pid, started_at, stopped_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               status = excluded.status,
               last_output_snippet = excluded.last_output_snippet,
               pid = excluded.pid,
               started_at = excluded.started_at,
               stopped_at = excluded.stopped_at,
               updated_at = excluded.updated_at",
        )
        .bind(&run.id)
        .bind(&run.team_member_id)
        .bind(run.status.as_str())
        .bind(&run.last_output_snippet)
        .bind(run.pid)
        .bind(run.started_at.map(|t| t.to_rfc3339()))
        .bind(run.stopped_at.map(|t| t.to_rfc3339()))
        .bind(run.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_agent_run_for_member(
        &self,
        team_member_id: &str,
    ) -> anyhow::Result<Option<AgentRun>> {
        let row = sqlx::query(
            "SELECT id, team_member_id, status, last_output_snippet, pid, started_at, stopped_at, updated_at
             FROM agent_runs WHERE team_member_id = ? ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(team_member_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| Self::row_to_agent_run(r)).transpose()
    }

    async fn get_claude_settings(&self) -> anyhow::Result<ClaudeSettings> {
        let row = sqlx::query(
            "SELECT credential_mode, api_key_ciphertext, api_base_url, updated_at
             FROM claude_settings WHERE id = ?",
        )
        .bind(settings_row_id())
        .fetch_optional(&self.pool)
        .await?;

        let Some(r) = row else {
            return Ok(ClaudeSettings {
                credential_mode: CredentialMode::CliLogin,
                api_key_ciphertext: None,
                api_base_url: None,
                updated_at: Utc::now(),
            });
        };

        let mode_str: String = r.get("credential_mode");
        Ok(ClaudeSettings {
            credential_mode: CredentialMode::parse(&mode_str)
                .ok_or_else(|| anyhow::anyhow!("invalid credential_mode: {mode_str}"))?,
            api_key_ciphertext: r.get("api_key_ciphertext"),
            api_base_url: r.get("api_base_url"),
            updated_at: Self::parse_dt(r.get::<String, _>("updated_at").as_str())?,
        })
    }

    async fn upsert_claude_settings(&self, settings: &ClaudeSettings) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO claude_settings (id, credential_mode, api_key_ciphertext, api_base_url, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               credential_mode = excluded.credential_mode,
               api_key_ciphertext = excluded.api_key_ciphertext,
               api_base_url = excluded.api_base_url,
               updated_at = excluded.updated_at",
        )
        .bind(settings_row_id())
        .bind(settings.credential_mode.as_str())
        .bind(&settings.api_key_ciphertext)
        .bind(&settings.api_base_url)
        .bind(settings.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl SqliteStore {
    fn row_to_task(r: sqlx::sqlite::SqliteRow) -> anyhow::Result<Task> {
        let status_str: String = r.get("status");
        let created_by_str: String = r.get("created_by");
        Ok(Task {
            id: r.get("id"),
            team_id: r.get("team_id"),
            title: r.get("title"),
            description: r.get("description"),
            status: TaskStatus::parse(&status_str)
                .ok_or_else(|| anyhow::anyhow!("invalid status: {status_str}"))?,
            assignee_member_id: r.get("assignee_member_id"),
            created_by: match created_by_str.as_str() {
                "human" => TaskActor::Human,
                "agent" => TaskActor::Agent,
                other => anyhow::bail!("invalid created_by: {other}"),
            },
            created_at: Self::parse_dt(r.get::<String, _>("created_at").as_str())?,
            updated_at: Self::parse_dt(r.get::<String, _>("updated_at").as_str())?,
        })
    }

    fn row_to_agent_run(r: sqlx::sqlite::SqliteRow) -> anyhow::Result<AgentRun> {
        let status_str: String = r.get("status");
        let started: Option<String> = r.get("started_at");
        let stopped: Option<String> = r.get("stopped_at");
        Ok(AgentRun {
            id: r.get("id"),
            team_member_id: r.get("team_member_id"),
            status: AgentRunStatus::parse(&status_str)
                .ok_or_else(|| anyhow::anyhow!("invalid run status: {status_str}"))?,
            last_output_snippet: r.get("last_output_snippet"),
            pid: r.get("pid"),
            started_at: started
                .map(|s| Self::parse_dt(&s))
                .transpose()?,
            stopped_at: stopped
                .map(|s| Self::parse_dt(&s))
                .transpose()?,
            updated_at: Self::parse_dt(r.get::<String, _>("updated_at").as_str())?,
        })
    }
}
