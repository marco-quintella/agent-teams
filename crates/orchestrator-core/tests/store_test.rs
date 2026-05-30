use orchestrator_core::{SqliteStore, Store, TaskActor, TaskStatus};

#[tokio::test]
async fn project_team_members_round_trip() {
    let store = temp_store().await;

    let project = store.create_project("/tmp/demo").await.unwrap();
    let team = store
        .create_team(&project.id, "alpha", "build the feature")
        .await
        .unwrap();

    store
        .add_team_member(&team.id, "lead", orchestrator_core::MemberRole::Lead, "coordinate")
        .await
        .unwrap();
    store
        .add_team_member(&team.id, "impl", orchestrator_core::MemberRole::Worker, "implement")
        .await
        .unwrap();

    let members = store.list_team_members(&team.id).await.unwrap();
    assert_eq!(members.len(), 2);
}

#[tokio::test]
async fn task_lifecycle_and_events() {
    let store = temp_store().await;
    let project = store.create_project("/tmp/demo").await.unwrap();
    let team = store
        .create_team(&project.id, "alpha", "")
        .await
        .unwrap();

    let task = store
        .create_task(
            &team.id,
            "Add README",
            "",
            TaskStatus::Backlog,
            None,
            TaskActor::Human,
        )
        .await
        .unwrap();

    let updated = store
        .update_task_status(&task.id, TaskStatus::InProgress, TaskActor::Human)
        .await
        .unwrap();
    assert_eq!(updated.status, TaskStatus::InProgress);

    let agent_task = store
        .create_task(
            &team.id,
            "Fix tests",
            "",
            TaskStatus::Backlog,
            None,
            TaskActor::Agent,
        )
        .await
        .unwrap();
    assert!(matches!(agent_task.created_by, TaskActor::Agent));
}

#[tokio::test]
async fn persistence_survives_reconnect() {
    let path = tempfile::NamedTempFile::new().unwrap();
    let url = format!("sqlite://{}", path.path().display());

    let team_id = {
        let store = SqliteStore::connect(&url).await.unwrap();
        let project = store.create_project("/tmp/demo").await.unwrap();
        let team = store.create_team(&project.id, "alpha", "").await.unwrap();
        store
            .add_team_member(
                &team.id,
                "lead",
                orchestrator_core::MemberRole::Lead,
                "",
            )
            .await
            .unwrap();
        team.id
    };

    let store = SqliteStore::connect(&url).await.unwrap();
    let members = store.list_team_members(&team_id).await.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name, "lead");
}

#[tokio::test]
async fn list_teams_includes_project_path() {
    let store = temp_store().await;
    let project = store.create_project("/tmp/v13-demo").await.unwrap();
    let team = store
        .create_team(&project.id, "Beta", "objective")
        .await
        .unwrap();

    let list = store.list_teams().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, team.id);
    assert_eq!(list[0].project_root_path, "/tmp/v13-demo");
    assert_eq!(list[0].status.as_str(), "stopped");
}

async fn temp_store() -> SqliteStore {
    SqliteStore::connect("sqlite::memory:").await.unwrap()
}
