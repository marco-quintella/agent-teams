use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent configuration data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Agent implementation kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentType {
    #[serde(alias = "ClaudeCode")]
    ClaudeCode,
    OpenRouter,
    ThirdParty(ThirdPartyConfig),
    Plugin(String),
}

/// Third-party provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyConfig {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

/// Workflow step input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepInput {
    pub prompt: Option<String>,
    pub context_vars: Option<Vec<String>>,
}

/// Condition expression evaluated against execution context (V1: stub evaluator).
pub type Condition = String;
