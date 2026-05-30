use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

use orchestrator_server::config::ServerConfig;

#[derive(Parser)]
#[command(name = "orchestrator-server")]
#[command(version, about = "Claude agent orchestrator control plane")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a workflow YAML file (validation only until engine ships)
    Run {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Validate a workflow YAML file
    Validate {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// List loaded plugins
    Plugins,
    /// Print build info and Claude CLI availability
    Doctor,
    /// Start the HTTP API and WebSocket server
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Run { file } => orchestrator_server::cli::run_workflow(file).await?,
        Commands::Validate { file } => orchestrator_server::cli::validate_workflow(file).await?,
        Commands::Plugins => orchestrator_server::cli::list_plugins().await?,
        Commands::Doctor => orchestrator_server::cli::doctor().await?,
        Commands::Serve => orchestrator_server::serve::run(ServerConfig::from_env()).await?,
    }

    Ok(())
}
