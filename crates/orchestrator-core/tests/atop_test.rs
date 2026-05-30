use orchestrator_core::atop::AtopIngestor;
use orchestrator_core::domain::{TaskActor, TaskStatus};
use orchestrator_core::events::EventBus;
use orchestrator_core::store::{SqliteStore, Store};
use tempfile::tempdir;

#[tokio::test]
async fn atop_task_create_persists_task() {
    let dir = tempdir().unwrap();
    let db = format!("sqlite:{}/test.db", dir.path().display());
    let store = SqliteStore::connect(&db).await.unwrap();
    let events = EventBus::default();

    let project = store.create_project(dir.path().to_str().unwrap()).await.unwrap();
    let team = store
        .create_team(&project.id, "Team", "Build the thing")
        .await
        .unwrap();

    let line = r#"{"op":"task.create","title":"Fix tests","status":"backlog"}"#;
    AtopIngestor::process_line(&store, &events, &team.id, line)
        .await
        .unwrap();

    let tasks = store.list_tasks(&team.id).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Fix tests");
    assert_eq!(tasks[0].status, TaskStatus::Backlog);
    assert_eq!(tasks[0].created_by, TaskActor::Agent);
}

#[tokio::test]
async fn atop_malformed_line_is_ignored() {
    let dir = tempdir().unwrap();
    let db = format!("sqlite:{}/test.db", dir.path().display());
    let store = SqliteStore::connect(&db).await.unwrap();
    let events = EventBus::default();

    let project = store.create_project(dir.path().to_str().unwrap()).await.unwrap();
    let team = store
        .create_team(&project.id, "Team", "Build the thing")
        .await
        .unwrap();

    AtopIngestor::process_line(&store, &events, &team.id, "not-json")
        .await
        .unwrap();

    assert!(store.list_tasks(&team.id).await.unwrap().is_empty());
}
