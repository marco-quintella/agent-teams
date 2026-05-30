use std::path::PathBuf;

use orchestrator_core::atop::ATOP_V1_SPEC;
use orchestrator_core::domain::{AgentRunStatus, MemberRole};
use orchestrator_core::events::EventBus;
use orchestrator_core::store::{SqliteStore, Store};
use orchestrator_core::claude_settings::LaunchEnv;
use orchestrator_core::supervisor::Supervisor;
use tempfile::tempdir;

#[cfg(windows)]
fn mock_echo() -> (PathBuf, Vec<String>) {
    (
        PathBuf::from("cmd.exe"),
        vec!["/C".to_string(), "echo".to_string(), "orchestrator-mock".to_string()],
    )
}

#[cfg(not(windows))]
fn mock_echo() -> (PathBuf, Vec<String>) {
    (
        PathBuf::from("/bin/echo"),
        vec!["orchestrator-mock".to_string()],
    )
}

#[tokio::test]
async fn supervisor_spawns_and_stops_mock_child() {
    let dir = tempdir().unwrap();
    let db = format!("sqlite:{}/test.db", dir.path().display());
    let store = SqliteStore::connect(&db).await.unwrap();
    let events = EventBus::default();
    let supervisor = Supervisor::new();

    let project = store.create_project(dir.path().to_str().unwrap()).await.unwrap();
    let team = store
        .create_team(&project.id, "Team", "Objective")
        .await
        .unwrap();
    let member = store
        .add_team_member(&team.id, "Worker", MemberRole::Worker, "Do work")
        .await
        .unwrap();

    let (cmd, args) = mock_echo();
    let run = supervisor
        .spawn_member(
            &store,
            &events,
            dir.path(),
            &team.id,
            &team.provisioning_prompt,
            &member,
            ATOP_V1_SPEC,
            Some((&cmd, &args)),
            0,
            LaunchEnv::default(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(run.status, AgentRunStatus::Running);

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    supervisor.stop_member(&store, &events, &member.id).await.unwrap();

    let stopped = store.get_agent_run_for_member(&member.id).await.unwrap().unwrap();
    assert_eq!(stopped.status, AgentRunStatus::Stopped);
}
