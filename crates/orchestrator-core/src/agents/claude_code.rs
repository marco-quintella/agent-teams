use async_trait::async_trait;
use duct::cmd;
use std::path::PathBuf;
use std::time::Instant;

use super::{Agent, AgentError, AgentInput, AgentOutput};

/// Agent that executes Claude Code CLI as a subprocess.
pub struct ClaudeCodeAgent {
    cli_path: PathBuf,
}

impl ClaudeCodeAgent {
    /// Creates an agent using `claude` or `claude-code` from PATH.
    pub fn new() -> anyhow::Result<Self> {
        let cli_path = which::which("claude")
            .or_else(|_| which::which("claude-code"))
            .map_err(|_| {
                anyhow::anyhow!("Claude CLI not found in PATH. Install Claude Code CLI first.")
            })?;

        Ok(Self { cli_path })
    }

    /// Returns true if the Claude CLI is available on PATH.
    pub fn is_available() -> bool {
        which::which("claude").is_ok() || which::which("claude-code").is_ok()
    }
}

#[async_trait]
impl Agent for ClaudeCodeAgent {
    async fn execute(&self, input: AgentInput) -> anyhow::Result<AgentOutput> {
        let started = Instant::now();
        let model = input
            .config
            .model
            .as_deref()
            .unwrap_or("claude-sonnet-4-20250514");

        let mut args = vec!["-p".to_string(), input.prompt.clone()];
        args.push("--model".to_string());
        args.push(model.to_string());

        if let Some(system_prompt) = &input.config.system_prompt {
            args.push("--system-prompt".to_string());
            args.push(system_prompt.clone());
        }

        let output = cmd(self.cli_path.as_os_str(), &args)
            .stdout_capture()
            .stderr_capture()
            .run()
            .map_err(|e| AgentError::ExecutionFailed(format!("Claude CLI failed: {e}")))?;

        Ok(AgentOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
            execution_time_ms: started.elapsed().as_millis() as u64,
        })
    }

    fn name(&self) -> &str {
        "ClaudeCode"
    }
}
