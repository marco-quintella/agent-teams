//! Core library for claude-orchestrator.

pub mod agents;
pub mod atop;
pub mod config;
pub mod domain;
pub mod error;
pub mod events;
pub mod orchestration;
pub mod plugins;
pub mod state;
pub mod store;
pub mod supervisor;

pub use error::OrchestratorError;

pub use agents::{Agent, AgentInput, AgentOutput, ClaudeCodeAgent};
pub use config::{
    AgentConfig, AgentDefinition, AgentType, Condition, StepInput, Workflow, WorkflowStep,
};
pub use orchestration::Orchestrator;
pub use domain::{
    AgentRun, AgentRunStatus, MemberRole, Project, Task, TaskActor, TaskEvent, TaskStatus, Team,
    TeamMember,
};
pub use state::ExecutionContext;
pub use atop::{AtopIngestor, AtopMessage, ATOP_V1_SPEC};
pub use events::{EventBus, OrchestratorEvent};
pub use store::{SqliteStore, Store, new_agent_run};
pub use supervisor::Supervisor;
