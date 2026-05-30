use serde::Deserialize;

use crate::domain::TaskStatus;

/// Parsed ATOP v1 line.
#[derive(Debug, Deserialize)]
#[serde(tag = "op")]
pub enum AtopMessage {
    #[serde(rename = "task.create")]
    TaskCreate {
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default = "default_backlog")]
        status: String,
        assignee: Option<String>,
    },
    #[serde(rename = "task.update_status")]
    TaskUpdateStatus {
        task_id: String,
        status: String,
    },
    #[serde(rename = "task.assign")]
    TaskAssign {
        task_id: String,
        assignee: Option<String>,
    },
    Ping,
}

fn default_backlog() -> String {
    "backlog".to_string()
}

impl AtopMessage {
    pub fn parse_status(s: &str) -> Option<TaskStatus> {
        TaskStatus::parse(s)
    }
}
