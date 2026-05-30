use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
};
use serde::Deserialize;

use orchestrator_core::domain::{Task, TaskActor, TaskStatus};
use orchestrator_core::events::OrchestratorEvent;
use orchestrator_core::Store;

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct CreateTaskBody {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_backlog")]
    pub status: String,
    pub assignee_member_id: Option<String>,
}

#[derive(Deserialize)]
pub struct PatchTaskBody {
    pub status: Option<String>,
    pub assignee_member_id: Option<Option<String>>,
}

fn default_backlog() -> String {
    "backlog".to_string()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/teams/{team_id}/tasks", get(list_tasks).post(create_task))
        .route("/teams/{team_id}/tasks/{task_id}", patch(patch_task))
}

async fn list_tasks(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }
    let tasks = state.store.list_tasks(&team_id).await.map_err(internal)?;
    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    Path(team_id): Path<String>,
    Json(body): Json<CreateTaskBody>,
) -> Result<Json<Task>, (StatusCode, String)> {
    if state.store.get_team(&team_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::NOT_FOUND, "team not found".into()));
    }

    let status = TaskStatus::parse(&body.status)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "invalid status".into()))?;

    let task = state
        .store
        .create_task(
            &team_id,
            &body.title,
            &body.description,
            status,
            body.assignee_member_id.as_deref(),
            TaskActor::Human,
        )
        .await
        .map_err(internal)?;

    state
        .events
        .publish(OrchestratorEvent::TaskUpdated { task: task.clone() });

    Ok(Json(task))
}

async fn patch_task(
    State(state): State<AppState>,
    Path((team_id, task_id)): Path<(String, String)>,
    Json(body): Json<PatchTaskBody>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let Some(task) = state.store.get_task(&task_id).await.map_err(internal)? else {
        return Err((StatusCode::NOT_FOUND, "task not found".into()));
    };
    if task.team_id != team_id {
        return Err((StatusCode::NOT_FOUND, "task not found".into()));
    }

    let mut updated = task;

    if let Some(status_str) = body.status {
        let status = TaskStatus::parse(&status_str)
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "invalid status".into()))?;
        updated = state
            .store
            .update_task_status(&task_id, status, TaskActor::Human)
            .await
            .map_err(internal)?;
    }

    if let Some(assignee) = body.assignee_member_id {
        updated = state
            .store
            .assign_task(&task_id, assignee.as_deref(), TaskActor::Human)
            .await
            .map_err(internal)?;
    }

    state.events.publish(OrchestratorEvent::TaskUpdated {
        task: updated.clone(),
    });

    Ok(Json(updated))
}

fn internal(e: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
