use serde::{Deserialize, Serialize};

use super::AgentConfig;

/// Agent definition within a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub agent_type: super::AgentType,
    pub config: AgentConfig,
}
