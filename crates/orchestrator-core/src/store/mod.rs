mod sqlite;

use chrono::Utc;
use async_trait::async_trait;
use crate::claude_settings::ClaudeSettings;
use crate::domain::{
    AgentRun, AgentRunStatus, MemberRole, Project, Task, TaskActor, TaskEvent, TaskStatus, Team,
    TeamMember, new_id,
};

pub use sqlite::SqliteStore;

/// Persistence boundary for orchestrator state.
#[async_trait]
pub trait Store: Send + Sync {
    async fn create_project(&self, root_path: &str) -> anyhow::Result<Project>;
    async fn get_project(&self, id: &str) -> anyhow::Result<Option<Project>>;

    async fn create_team(
        &self,
        project_id: &str,
        name: &str,
        provisioning_prompt: &str,
    ) -> anyhow::Result<Team>;

    async fn add_team_member(
        &self,
        team_id: &str,
        name: &str,
        role: MemberRole,
        role_prompt: &str,
    ) -> anyhow::Result<TeamMember>;

    async fn list_team_members(&self, team_id: &str) -> anyhow::Result<Vec<TeamMember>>;

    async fn get_team(&self, team_id: &str) -> anyhow::Result<Option<Team>>;

    async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Task>>;

    async fn create_task(
        &self,
        team_id: &str,
        title: &str,
        description: &str,
        status: TaskStatus,
        assignee_member_id: Option<&str>,
        created_by: TaskActor,
    ) -> anyhow::Result<Task>;

    async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        actor: TaskActor,
    ) -> anyhow::Result<Task>;

    async fn assign_task(
        &self,
        task_id: &str,
        assignee_member_id: Option<&str>,
        actor: TaskActor,
    ) -> anyhow::Result<Task>;

    async fn list_tasks(&self, team_id: &str) -> anyhow::Result<Vec<Task>>;

    async fn record_task_event(
        &self,
        task_id: &str,
        actor: TaskActor,
        event_type: &str,
        payload_json: &str,
    ) -> anyhow::Result<TaskEvent>;

    async fn upsert_agent_run(&self, run: &AgentRun) -> anyhow::Result<()>;

    async fn get_agent_run_for_member(
        &self,
        team_member_id: &str,
    ) -> anyhow::Result<Option<AgentRun>>;

    async fn get_claude_settings(&self) -> anyhow::Result<ClaudeSettings>;

    async fn upsert_claude_settings(&self, settings: &ClaudeSettings) -> anyhow::Result<()>;
}

/// Creates a new agent run row for a team member.
pub fn new_agent_run(team_member_id: &str) -> AgentRun {
    let now = Utc::now();
    AgentRun {
        id: new_id(),
        team_member_id: team_member_id.to_string(),
        status: AgentRunStatus::Starting,
        last_output_snippet: String::new(),
        pid: None,
        started_at: Some(now),
        stopped_at: None,
        updated_at: now,
    }
}
