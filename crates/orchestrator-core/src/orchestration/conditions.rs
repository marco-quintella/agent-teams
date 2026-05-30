use anyhow::Result;

use crate::state::ExecutionContext;

/// Evaluates a condition string against context (V1 stub — always false).
pub fn evaluate_condition(_ctx: &ExecutionContext, condition: &str) -> Result<bool> {
    tracing::debug!(condition, "condition evaluation not implemented");
    Ok(false)
}
