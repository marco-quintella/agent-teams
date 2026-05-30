use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agents::AgentOutput;

/// Shared execution context for workflow runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub outputs: HashMap<String, AgentOutput>,
}
