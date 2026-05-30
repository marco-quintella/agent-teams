use serde::{Deserialize, Serialize};

use super::{AgentDefinition, WorkflowStep};

/// Workflow definition loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub agents: Vec<AgentDefinition>,
    pub steps: Vec<WorkflowStep>,
}
