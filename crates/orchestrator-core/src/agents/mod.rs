mod agent;
mod api_client;
mod claude_code;
mod traits;

pub use agent::PluginAgent;
pub use api_client::{ApiClientAgent, ApiClientConfig};
pub use claude_code::ClaudeCodeAgent;
pub use traits::{Agent, AgentError, AgentInput, AgentOutput};
