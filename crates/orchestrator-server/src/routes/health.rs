use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub version: &'static str,
    pub profile: String,
    pub claude_on_path: bool,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        version: env!("CARGO_PKG_VERSION"),
        profile: state.config.profile.as_str().to_string(),
        claude_on_path: state.supervisor.claude_available(),
    })
}
