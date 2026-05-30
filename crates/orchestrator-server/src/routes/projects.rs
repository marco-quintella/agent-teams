use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;

use orchestrator_core::Store;

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct CreateProjectBody {
    pub root_path: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/projects", post(create_project))
}

async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectBody>,
) -> Result<Json<orchestrator_core::Project>, (StatusCode, String)> {
    AppState::validate_project_path(&body.root_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let project = state
        .store
        .create_project(&body.root_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(project))
}
