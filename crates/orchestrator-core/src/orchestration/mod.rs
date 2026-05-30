mod conditions;

use crate::config::Workflow;
use crate::error::OrchestratorError;

/// Workflow orchestration engine (execution deferred past V1).
pub struct Orchestrator;

impl Orchestrator {
    pub fn new() -> Self {
        Self
    }

    /// Validates and loads a workflow; full execution is follow-up work.
    pub async fn execute(&self, workflow: &Workflow) -> anyhow::Result<()> {
        tracing::info!(
            workflow = %workflow.name,
            steps = workflow.steps.len(),
            "workflow execution not yet implemented"
        );
        Err(OrchestratorError::WorkflowNotImplemented.into())
    }
}
