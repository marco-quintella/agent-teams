//! Core library for claude-orchestrator.

pub mod agents;
pub mod config;
pub mod domain;
pub mod error;
pub mod orchestration;
pub mod plugins;
pub mod state;
pub mod store;

pub use error::OrchestratorError;

pub use agents::{Agent, AgentInput, AgentOutput};
pub use config::{
    AgentConfig, AgentDefinition, AgentType, Condition, StepInput, Workflow, WorkflowStep,
};
pub use orchestration::Orchestrator;
pub use domain::{
    AgentRun, AgentRunStatus, MemberRole, Project, Task, TaskActor, TaskEvent, TaskStatus, Team,
    TeamMember,
};
pub use state::ExecutionContext;
pub use store::{SqliteStore, Store, new_agent_run};
