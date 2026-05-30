use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use super::{Agent, AgentError, AgentInput, AgentOutput};

/// Configuration for third-party API clients (deferred for V1 product path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

/// HTTP LLM agent (not used in Claude-only V1 UI).
pub struct ApiClientAgent {
    config: ApiClientConfig,
    client: Client,
    name: String,
}

impl ApiClientAgent {
    pub fn new(config: ApiClientConfig) -> Self {
        let name = format!("API-{}", config.provider);
        Self {
            config,
            client: Client::new(),
            name,
        }
    }
}

#[async_trait]
impl Agent for ApiClientAgent {
    async fn execute(&self, input: AgentInput) -> anyhow::Result<AgentOutput> {
        let started = Instant::now();
        let endpoint = self
            .config
            .endpoint
            .clone()
            .unwrap_or_else(|| match self.config.provider.as_str() {
                "OpenRouter" => "https://openrouter.ai/api/v1/chat/completions".to_string(),
                _ => "https://api.openai.com/v1/chat/completions".to_string(),
            });

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": [{ "role": "user", "content": input.prompt }],
            "stream": false
        });

        if let Some(temp) = self.config.parameters.get("temperature").and_then(|v| v.as_f64()) {
            body["temperature"] = serde_json::json!(temp);
        }

        let mut request = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json");

        if let Some(key) = &self.config.api_key {
            request = request.bearer_auth(key);
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentError::ExecutionFailed(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AgentError::ExecutionFailed(e.to_string()))?;

        if !status.is_success() {
            return Err(AgentError::ExecutionFailed(format!(
                "HTTP {}: {}",
                status.as_u16(),
                text
            ))
            .into());
        }

        let stdout = match serde_json::from_str::<ApiResponse>(&text) {
            Ok(parsed) => parsed
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default(),
            Err(_) => text,
        };

        Ok(AgentOutput {
            stdout,
            stderr: String::new(),
            exit_code: Some(0),
            execution_time_ms: started.elapsed().as_millis() as u64,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
