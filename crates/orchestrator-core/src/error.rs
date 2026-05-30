//! Shared error types.

use thiserror::Error;

/// Core orchestrator errors.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("workflow execution is not implemented yet")]
    WorkflowNotImplemented,
}
