use axum::body::Body;
use http_body_util::BodyExt;
use orchestrator_core::events::{EventBus, OrchestratorEvent};
use orchestrator_core::store::SqliteStore;
use orchestrator_core::supervisor::Supervisor;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use tower::ServiceExt;

use orchestrator_server::app_state::AppState;
use orchestrator_server::config::{Profile, ServerConfig};
use orchestrator_server::routes;

async fn test_state(dir: &tempfile::TempDir) -> AppState {
    let config = ServerConfig {
        profile: Profile::Dev,
        bind_addr: "127.0.0.1".into(),
        port: 0,
        data_dir: dir.path().to_path_buf(),
        static_dir: None,
    };
    let db_url = config.database_url();
    let store = SqliteStore::connect(&db_url).await.unwrap();
    AppState {
        config: Arc::new(config),
        store: Arc::new(store),
        supervisor: Arc::new(Supervisor::new()),
        events: EventBus::default(),
    }
}

#[tokio::test]
async fn create_task_emits_ws_payload_shape() {
    let dir = tempdir().unwrap();
    let state = test_state(&dir).await;
    let app = routes::build_app(state.clone());

    let project_body = json!({ "root_path": dir.path().to_str().unwrap() });
    let project_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&project_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(project_resp.status(), 200);

    let project_json: serde_json::Value =
        serde_json::from_slice(&project_resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let project_id = project_json["id"].as_str().unwrap();

    let team_body = json!({
        "project_id": project_id,
        "name": "Alpha",
        "provisioning_prompt": "Ship V1"
    });
    let team_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/teams")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&team_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(team_resp.status(), 200);
    let team_json: serde_json::Value =
        serde_json::from_slice(&team_resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let team_id = team_json["id"].as_str().unwrap();

    let mut events = state.events.subscribe();

    let task_body = json!({ "title": "Kanban card", "status": "backlog" });
    let task_resp = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/teams/{team_id}/tasks"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&task_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_resp.status(), 200);

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), events.recv())
        .await
        .expect("timed out waiting for event")
        .unwrap();

    match event {
        OrchestratorEvent::TaskUpdated { task } => {
            assert_eq!(task.title, "Kanban card");
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[tokio::test]
async fn project_path_with_dotdot_is_rejected() {
    let dir = tempdir().unwrap();
    let state = test_state(&dir).await;
    let app = routes::build_app(state);

    let body = json!({ "root_path": "../escape" });
    let resp = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}
