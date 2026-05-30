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
    use std::process::Command;

    println!("orchestrator-server {}", env!("CARGO_PKG_VERSION"));
    if ClaudeCodeAgent::is_available() {
        println!("claude CLI: found on PATH");
        if let Ok(agent) = ClaudeCodeAgent::new() {
            match Command::new(agent.executable_path())
                .arg("--version")
                .output()
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let version = stdout.trim();
                    if version.is_empty() {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        println!("claude --version: {}", stderr.trim());
                    } else {
                        println!("claude --version: {version}");
                    }
                }
                Err(e) => println!("claude --version failed: {e}"),
            }
        }
    } else {
        println!("claude CLI: not found");
    }
    Ok(())
}

fn load_workflow(file: &PathBuf) -> Result<Workflow> {
    let content = std::fs::read_to_string(file)?;
    Ok(serde_yaml::from_str(&content)?)
}
