use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::AgentConfig;

/// Input provided to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub prompt: String,
    pub config: AgentConfig,
}

/// Output produced by an agent after execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u64,
}

/// Core trait for agent implementations.
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, input: AgentInput) -> anyhow::Result<AgentOutput>;

    fn name(&self) -> &str {
        "unnamed"
    }

    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Error types for agent operations.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("streaming not supported")]
    StreamingNotSupported,

    #[error("unsupported agent type")]
    UnsupportedAgentType,
}
