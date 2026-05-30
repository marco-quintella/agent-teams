use anyhow::Result;
use std::path::PathBuf;

use orchestrator_core::{ClaudeCodeAgent, OrchestratorError, Workflow};

pub async fn run_workflow(file: PathBuf) -> Result<()> {
    let workflow = load_workflow(&file)?;
    tracing::info!(name = %workflow.name, "loaded workflow");

    let engine = orchestrator_core::Orchestrator::new();
    match engine.execute(&workflow).await {
        Err(e) if e.downcast_ref::<OrchestratorError>().is_some() => {
            println!(
                "Workflow '{}' is valid; execution is not implemented yet.",
                workflow.name
            );
            Ok(())
        }
        other => other,
    }
}

pub async fn validate_workflow(file: PathBuf) -> Result<()> {
    let workflow = load_workflow(&file)?;
    println!("Workflow '{}' (v{}) is valid.", workflow.name, workflow.version);
    Ok(())
}

pub async fn list_plugins() -> Result<()> {
    println!("No plugins loaded.");
    Ok(())
}

pub async fn doctor() -> Result<()> {
    println!("orchestrator-server {}", env!("CARGO_PKG_VERSION"));
    println!(
        "claude CLI: {}",
        if ClaudeCodeAgent::is_available() {
            "found on PATH"
        } else {
            "not found"
        }
    );
    Ok(())
}

fn load_workflow(file: &PathBuf) -> Result<Workflow> {
    let content = std::fs::read_to_string(file)?;
    Ok(serde_yaml::from_str(&content)?)
}
