use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kanban column / task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Backlog,
    InProgress,
    Review,
    Done,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::InProgress => "in_progress",
            Self::Review => "review",
            Self::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "backlog" => Some(Self::Backlog),
            "in_progress" => Some(Self::InProgress),
            "review" => Some(Self::Review),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Lead,
    Worker,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lead => "lead",
            Self::Worker => "worker",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "lead" => Some(Self::Lead),
            "worker" => Some(Self::Worker),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskActor {
    Human,
    Agent,
}

impl TaskActor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Starting,
    Running,
    Idle,
    Error,
    Stopped,
}

impl AgentRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Idle => "idle",
            Self::Error => "error",
            Self::Stopped => "stopped",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "starting" => Some(Self::Starting),
            "running" => Some(Self::Running),
            "idle" => Some(Self::Idle),
            "error" => Some(Self::Error),
            "stopped" => Some(Self::Stopped),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub root_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub provisioning_prompt: String,
    pub created_at: DateTime<Utc>,
}

/// Team row for launcher history (includes project path and run status).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSummary {
    pub id: String,
    pub name: String,
    pub project_root_path: String,
    pub created_at: DateTime<Utc>,
    pub status: TeamRunStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamRunStatus {
    Running,
    Stopped,
}

impl TeamRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub team_id: String,
    pub name: String,
    pub role: MemberRole,
    pub role_prompt: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub team_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub assignee_member_id: Option<String>,
    pub created_by: TaskActor,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub id: String,
    pub task_id: String,
    pub actor: TaskActor,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub team_member_id: String,
    pub status: AgentRunStatus,
    pub last_output_snippet: String,
    pub pid: Option<i64>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}
