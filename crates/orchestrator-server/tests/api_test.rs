use axum::body::Body;
use http_body_util::BodyExt;
use orchestrator_core::events::OrchestratorEvent;
use serde_json::json;
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
    AppState::new(config).await.unwrap()
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

#[tokio::test]
async fn setup_doctor_and_settings_roundtrip() {
    let dir = tempdir().unwrap();
    let state = test_state(&dir).await;
    let app = routes::build_app(state.clone());

    let doctor_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/setup/doctor")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(doctor_resp.status(), 200);
    let doctor_json: serde_json::Value =
        serde_json::from_slice(&doctor_resp.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    assert!(doctor_json["cli"].is_object());
    assert!(doctor_json["credentials"].is_object());

    let master = orchestrator_core::load_or_create_master_key(dir.path()).unwrap();
    let ct = orchestrator_core::encrypt_api_key("sk-test-key-for-api", &master).unwrap();

    let patch_body = json!({
        "credential_mode": "api_key",
        "api_key": "sk-test-key-for-api",
        "api_base_url": "https://openrouter.ai/api"
    });
    let patch_resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri("/api/setup/claude-settings")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&patch_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_resp.status(), 200);
    let _ = ct;

    let doctor2 = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/setup/doctor")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let doctor2_json: serde_json::Value =
        serde_json::from_slice(&doctor2.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(doctor2_json["credentials"]["ready"], true);
}

#[tokio::test]
async fn serves_spa_when_dist_present() {
    let dir = tempdir().unwrap();
    let dist = dir.path().join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    std::fs::write(dist.join("index.html"), "<html><body>ui</body></html>").unwrap();

    let config = ServerConfig {
        profile: Profile::Dev,
        bind_addr: "127.0.0.1".into(),
        port: 0,
        data_dir: dir.path().join("data"),
        static_dir: Some(dist),
    };
    let state = AppState::new(config).await.unwrap();
    let app = routes::build_app(state);

    let resp = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
