use async_trait::async_trait;

use super::{Agent, AgentInput, AgentOutput};

/// Placeholder agent for plugin-backed roles (not wired in V1).
pub struct PluginAgent {
    name: String,
}

impl PluginAgent {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl Agent for PluginAgent {
    async fn execute(&self, _input: AgentInput) -> anyhow::Result<AgentOutput> {
        Ok(AgentOutput {
            stdout: format!("Placeholder response from {}", self.name),
            stderr: String::new(),
            exit_code: Some(0),
            execution_time_ms: 0,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
