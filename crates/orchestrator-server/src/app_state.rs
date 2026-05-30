use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use orchestrator_core::atop::{AtopIngestor, ATOP_V1_SPEC};
use orchestrator_core::events::EventBus;
use orchestrator_core::store::{SqliteStore, Store};
use orchestrator_core::supervisor::{format_objective_envelope, Supervisor, MemberWorkspace};
use tokio_util::sync::CancellationToken;

use crate::config::ServerConfig;

/// Shared application state for HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub store: Arc<SqliteStore>,
    pub supervisor: Arc<Supervisor>,
    pub events: EventBus,
    snippet_refresh_cancel: Arc<DashMap<String, CancellationToken>>,
}

impl AppState {
    pub async fn new(config: ServerConfig) -> anyhow::Result<Self> {
        let store = SqliteStore::connect(&config.database_url()).await?;
        Ok(Self {
            config: Arc::new(config),
            store: Arc::new(store),
            supervisor: Arc::new(Supervisor::new()),
            events: EventBus::default(),
            snippet_refresh_cancel: Arc::new(DashMap::new()),
        })
    }

    pub fn validate_project_path(path: &str) -> Result<(), String> {
        if path.contains("..") {
            return Err("path must not contain '..'".into());
        }
        if path.trim().is_empty() {
            return Err("path must not be empty".into());
        }
        Ok(())
    }

    fn stop_snippet_refresh(&self, team_id: &str) {
        if let Some((_, token)) = self.snippet_refresh_cancel.remove(team_id) {
            token.cancel();
        }
    }

    fn start_snippet_refresh(&self, team_id: &str, member_ids: Vec<String>) {
        self.stop_snippet_refresh(team_id);

        let token = CancellationToken::new();
        self.snippet_refresh_cancel
            .insert(team_id.to_string(), token.clone());

        let supervisor = Arc::clone(&self.supervisor);
        let store = Arc::clone(&self.store);
        let events = self.events.clone();
        let team_id_owned = team_id.to_string();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            loop {
                tokio::select! {
                    () = token.cancelled() => break,
                    _ = interval.tick() => {
                        if !supervisor.has_live_sessions(&member_ids) {
                            break;
                        }
                        for member_id in &member_ids {
                            if supervisor.has_live_session(member_id) {
                                let _ = supervisor
                                    .refresh_snippets(store.as_ref(), &events, member_id)
                                    .await;
                            }
                        }
                    }
                }
            }
            tracing::debug!(team_id = %team_id_owned, "snippet refresh loop stopped");
        });
    }

    pub async fn launch_team(&self, team_id: &str) -> Result<(), LaunchError> {
        let team = self
            .store
            .get_team(team_id)
            .await
            .map_err(LaunchError::Internal)?
            .ok_or(LaunchError::NotFound)?;

        let project = self
            .store
            .get_project(&team.project_id)
            .await
            .map_err(LaunchError::Internal)?
            .ok_or(LaunchError::NotFound)?;

        Self::validate_project_path(&project.root_path).map_err(LaunchError::BadRequest)?;

        let members = self
            .store
            .list_team_members(team_id)
            .await
            .map_err(LaunchError::Internal)?;

        if members.is_empty() {
            return Err(LaunchError::BadRequest("team has no members".into()));
        }

        let member_ids: Vec<String> = members.iter().map(|m| m.id.clone()).collect();
        if self.supervisor.has_live_sessions(&member_ids) {
            return Err(LaunchError::Conflict);
        }

        // Stop orphaned sessions from prior teams (same operator, new launch).
        self.supervisor
            .stop_all_sessions(self.store.as_ref(), &self.events)
            .await
            .map_err(LaunchError::Internal)?;

        let project_root = Path::new(&project.root_path);
        std::fs::create_dir_all(project_root).map_err(|e| LaunchError::Internal(e.into()))?;

        let tasks = self
            .store
            .list_tasks(team_id)
            .await
            .map_err(LaunchError::Internal)?;
        let task_count = tasks.len();

        for member in &members {
            let ws = MemberWorkspace::new(project_root, team_id, &member.id);
            ws.ensure().await.map_err(|e| LaunchError::Internal(e.into()))?;

            self.supervisor
                .spawn_member(
                    self.store.as_ref(),
                    &self.events,
                    project_root,
                    team_id,
                    &team.provisioning_prompt,
                    member,
                    ATOP_V1_SPEC,
                    None,
                    task_count,
                )
                .await
                .map_err(|e| {
                    if e.to_string().contains("Claude CLI not found") {
                        LaunchError::ClaudeMissing
                    } else {
                        LaunchError::Internal(e)
                    }
                })?;

            let store = Arc::clone(&self.store);
            let events = self.events.clone();
            let protocol = ws.protocol_file.clone();
            let tid = team_id.to_string();
            tokio::spawn(async move {
                AtopIngestor::run_loop(store.as_ref(), &events, &tid, protocol).await;
            });
        }

        self.start_snippet_refresh(team_id, member_ids);

        self.events
            .publish(orchestrator_core::OrchestratorEvent::TeamUpdated {
                team_id: team_id.to_string(),
            });

        Ok(())
    }

    pub async fn stop_team(&self, team_id: &str) -> anyhow::Result<()> {
        self.stop_snippet_refresh(team_id);
        let members = self.store.list_team_members(team_id).await?;
        self.supervisor
            .stop_team(self.store.as_ref(), &self.events, &members)
            .await?;
        self.events
            .publish(orchestrator_core::OrchestratorEvent::TeamUpdated {
                team_id: team_id.to_string(),
            });
        Ok(())
    }

    pub async fn deliver_message(&self, team_id: &str, text: &str) -> anyhow::Result<()> {
        let members = self.store.list_team_members(team_id).await?;
        let tasks = self.store.list_tasks(team_id).await?;
        let envelope = format_objective_envelope(&tasks, text);
        let supervisor = Arc::clone(&self.supervisor);

        tokio::task::spawn_blocking(move || supervisor.deliver_lead_message(&members, &envelope))
            .await
            .map_err(|e| anyhow::anyhow!("message delivery task join failed: {e}"))??;

        Ok(())
    }
}

#[derive(Debug)]
pub enum LaunchError {
    NotFound,
    Conflict,
    ClaudeMissing,
    BadRequest(String),
    Internal(anyhow::Error),
}

impl LaunchError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::ClaudeMissing => StatusCode::SERVICE_UNAVAILABLE,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::NotFound => "team or project not found".into(),
            Self::Conflict => "team already launched".into(),
            Self::ClaudeMissing => "claude CLI not found on PATH".into(),
            Self::BadRequest(msg) => msg.clone(),
            Self::Internal(e) => e.to_string(),
        }
    }
}
