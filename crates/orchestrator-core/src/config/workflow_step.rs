use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Condition, StepInput};

/// A single step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowStep {
    Run {
        agent: String,
        input: Option<StepInput>,
        output_var: Option<String>,
    },
    Parallel {
        steps: Vec<WorkflowStep>,
        output_vars: Option<HashMap<String, String>>,
    },
    Conditional {
        condition: Condition,
        true_steps: Vec<WorkflowStep>,
        false_steps: Option<Vec<WorkflowStep>>,
    },
    While {
        condition: Condition,
        steps: Vec<WorkflowStep>,
        max_iterations: Option<usize>,
    },
    ForEach {
        items: String,
        item_var: String,
        steps: Vec<WorkflowStep>,
    },
}
