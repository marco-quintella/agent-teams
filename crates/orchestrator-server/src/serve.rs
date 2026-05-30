use axum::Router;
use tracing::info;

use crate::app_state::AppState;
use crate::config::ServerConfig;
use crate::routes;

pub async fn run(config: ServerConfig) -> anyhow::Result<()> {
    let state = AppState::new(config).await?;
    let addr = state.config.socket_addr()?;
    let profile = state.config.profile.as_str();
    let app: Router = routes::build_app(state);

    info!(%addr, %profile, "orchestrator API listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
