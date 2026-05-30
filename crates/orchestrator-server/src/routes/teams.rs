use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;

use orchestrator_core::domain::{AgentRun, MemberRole, Team, TeamMember, TeamSummary};
use orchestrator_core::Store;

use crate::app_state::{AppState, LaunchError, MessageError};

#[derive(Deserialize)]
pub struct CreateTeamBody {
    pub project_id: String,
    pub name: String,
    pub provisioning_prompt: String,
}

#[derive(Deserialize)]
pub struct AddMemberBody {
    pub name: String,
    pub role: String,
    pub role_prompt: String,
}

#[derive(Deserialize)]
pub struct TeamMessageBody {
    pub text: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/teams", get(list_teams).post(create_team))
        .route("/teams/{team_id}/members", post(add_member))
        .route("/teams/{team_id}/launch", post(launch_team))
        .route("/teams/{team_id}/stop", post(stop_team))
        .route("/teams/{team_id}/message", post(team_message))
        .route("/teams/{team_id}/agent-runs", get(list_agent_runs))
        .route("/teams/{team_id}/members", get(list_members))
}

async fn list_teams(State(state): State<AppState>) -> Result<Json<Vec<TeamSummary>>, (StatusCode, String)> {
    let teams = state.store.list_teams().await.map_err(internal)?;
    Ok(Json(teams))
}

async fn list_members(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
) -> Result<Json<Vec<TeamMember>>, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }
    let members = state.store.list_team_members(&team_id).await.map_err(internal)?;
    Ok(Json(members))
}

async fn list_agent_runs(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
) -> Result<Json<Vec<AgentRun>>, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }
    let members = state.store.list_team_members(&team_id).await.map_err(internal)?;
    let mut runs = Vec::new();
    for member in members {
        if let Some(run) = state
            .store
            .get_agent_run_for_member(&member.id)
            .await
            .map_err(internal)?
        {
            runs.push(run);
        }
    }
    Ok(Json(runs))
}

async fn create_team(
    State(state): State<AppState>,
    Json(body): Json<CreateTeamBody>,
) -> Result<Json<Team>, (StatusCode, String)> {
    if state.store.get_project(&body.project_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".into()));
    }

    let team = state
        .store
        .create_team(&body.project_id, &body.name, &body.provisioning_prompt)
        .await
        .map_err(internal)?;

    Ok(Json(team))
}

async fn add_member(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
    Json(body): Json<AddMemberBody>,
) -> Result<Json<TeamMember>, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }

    let role = MemberRole::parse(&body.role)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "invalid role".into()))?;

    let member = state
        .store
        .add_team_member(&team_id, &body.name, role, &body.role_prompt)
        .await
        .map_err(internal)?;

    Ok(Json(member))
}

async fn launch_team(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.launch_team(&team_id).await.map_err(|e| match e {
        LaunchError::NotFound => (StatusCode::NOT_FOUND, e.message()),
        LaunchError::Conflict => (StatusCode::CONFLICT, e.message()),
        LaunchError::ClaudeMissing => (StatusCode::SERVICE_UNAVAILABLE, e.message()),
        LaunchError::CredentialsNotConfigured => (StatusCode::BAD_REQUEST, e.message()),
        LaunchError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        LaunchError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn stop_team(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }
    state.stop_team(&team_id).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn team_message(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
    Json(body): Json<TeamMessageBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }
    state.deliver_message(&team_id, &body.text).await.map_err(|e| match &e {
        MessageError::LeadSessionNotRunning(msg) => (e.status_code(), msg.clone()),
        MessageError::Other(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

fn internal(e: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
