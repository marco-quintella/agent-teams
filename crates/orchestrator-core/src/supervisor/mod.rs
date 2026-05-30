mod bootstrap;
mod session;
mod workspace;

use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;

use crate::agents::ClaudeCodeAgent;
use crate::domain::{AgentRun, AgentRunStatus, MemberRole, TeamMember};
use crate::events::{EventBus, OrchestratorEvent};
use crate::claude_settings::LaunchEnv;
use crate::store::{Store, new_agent_run};

pub use bootstrap::{build_role_markdown, claude_spawn_args, format_objective_envelope, format_session_bootstrap};
pub use session::MemberSession;
pub use workspace::MemberWorkspace;

/// Manages Claude Code (or mock) child processes per team member.
pub struct Supervisor {
    sessions: DashMap<String, Arc<MemberSession>>,
    claude_path: Option<PathBuf>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            claude_path: which::which("claude")
                .ok()
                .or_else(|| which::which("claude-code").ok()),
        }
    }

    pub fn claude_available(&self) -> bool {
        self.claude_path.is_some()
    }

    pub fn has_live_session(&self, member_id: &str) -> bool {
        self.sessions.contains_key(member_id)
    }

    /// Spawns a teammate process and records agent run state.
    pub async fn spawn_member<S: Store + ?Sized>(
        &self,
        store: &S,
        events: &EventBus,
        project_root: &Path,
        team_id: &str,
        team_prompt: &str,
        member: &TeamMember,
        atop_spec: &str,
        mock_command: Option<(&Path, &[String])>,
        task_count: usize,
        launch_env: LaunchEnv,
    ) -> anyhow::Result<AgentRun> {
        if self.sessions.contains_key(&member.id) {
            anyhow::bail!("member {} already has a live session", member.id);
        }

        let mut run = new_agent_run(&member.id);
        run.status = AgentRunStatus::Starting;
        store.upsert_agent_run(&run).await?;

        let workspace = MemberWorkspace::new(project_root, team_id, &member.id);
        let role_md = build_role_markdown(member, team_prompt, atop_spec, &workspace);

        let (cmd_path, args): (PathBuf, Vec<String>) = if let Some((path, args)) = mock_command {
            (path.to_path_buf(), args.to_vec())
        } else {
            let path = self
                .claude_path
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Claude CLI not found on PATH"))?;
            let spawn_args = claude_spawn_args(&workspace.role_file, project_root);
            (path, spawn_args)
        };

        let project_root = project_root.to_path_buf();
        let team_id = team_id.to_string();
        let member_id = member.id.clone();
        let bootstrap = format_session_bootstrap(&workspace, task_count);
        let env_pairs: Vec<(String, String)> = launch_env
            .vars
            .into_iter()
            .collect();

        let session = tokio::task::spawn_blocking(move || {
            let session = MemberSession::spawn(
                &project_root,
                &team_id,
                &member_id,
                &cmd_path,
                &args,
                &role_md,
                &env_pairs,
            )?;
            session.write_stdin(&bootstrap)?;
            Ok::<Arc<MemberSession>, anyhow::Error>(Arc::new(session))
        })
        .await
        .map_err(|e| anyhow::anyhow!("spawn task join failed: {e}"))??;

        run.status = AgentRunStatus::Running;
        run.pid = None;
        run.last_output_snippet = session.last_output_snippet().await;
        self.sessions.insert(member.id.clone(), session);
        run.updated_at = Utc::now();
        store.upsert_agent_run(&run).await?;
        events.publish(OrchestratorEvent::AgentRunUpdated { run: run.clone() });

        Ok(run)
    }

    pub async fn stop_member<S: Store + ?Sized>(
        &self,
        store: &S,
        events: &EventBus,
        member_id: &str,
    ) -> anyhow::Result<()> {
        if let Some((_, session)) = self.sessions.remove(member_id) {
            session.stop();
        }

        if let Some(mut run) = store.get_agent_run_for_member(member_id).await? {
            run.status = AgentRunStatus::Stopped;
            run.stopped_at = Some(Utc::now());
            run.updated_at = Utc::now();
            store.upsert_agent_run(&run).await?;
            events.publish(OrchestratorEvent::AgentRunUpdated { run });
        }
        Ok(())
    }

    pub async fn stop_team<S: Store + ?Sized>(
        &self,
        store: &S,
        events: &EventBus,
        members: &[TeamMember],
    ) -> anyhow::Result<()> {
        for member in members {
            self.stop_member(store, events, &member.id).await?;
        }
        Ok(())
    }

    pub async fn refresh_snippets<S: Store + ?Sized>(
        &self,
        store: &S,
        events: &EventBus,
        member_id: &str,
    ) -> anyhow::Result<()> {
        let Some(session) = self.sessions.get(member_id) else {
            return Ok(());
        };
        if !session.is_alive() {
            drop(session);
            self.sessions.remove(member_id);
            if let Some(mut run) = store.get_agent_run_for_member(member_id).await? {
                run.status = AgentRunStatus::Error;
                run.stopped_at = Some(Utc::now());
                run.updated_at = Utc::now();
                store.upsert_agent_run(&run).await?;
                events.publish(OrchestratorEvent::AgentRunUpdated { run });
            }
            return Ok(());
        }
        if let Some(mut run) = store.get_agent_run_for_member(member_id).await? {
            run.last_output_snippet = session.last_output_snippet().await;
            run.updated_at = Utc::now();
            store.upsert_agent_run(&run).await?;
            events.publish(OrchestratorEvent::AgentRunUpdated { run });
        }
        Ok(())
    }

    pub fn is_claude_available(&self) -> bool {
        ClaudeCodeAgent::is_available()
    }

    /// Delivers text to the lead member's live PTY session.
    pub fn deliver_lead_message(
        &self,
        members: &[TeamMember],
        text: &str,
    ) -> anyhow::Result<()> {
        let lead = members
            .iter()
            .find(|m| m.role == MemberRole::Lead)
            .ok_or_else(|| anyhow::anyhow!("team has no lead member"))?;
        let session = self
            .sessions
            .get(&lead.id)
            .ok_or_else(|| anyhow::anyhow!("lead session is not running"))?;
        if !session.is_alive() {
            anyhow::bail!("lead session is not running (child process exited)");
        }
        session.write_stdin(text).map_err(|e| {
            if e.to_string().contains("closed") {
                anyhow::anyhow!("lead session is not running (PTY stdin closed)")
            } else {
                e
            }
        })?;
        Ok(())
    }

    pub fn has_live_sessions(&self, member_ids: &[String]) -> bool {
        member_ids.iter().any(|id| self.sessions.contains_key(id))
    }

    /// Stops every live session (used before launch to avoid orphaned Claude processes).
    pub async fn stop_all_sessions<S: Store + ?Sized>(
        &self,
        store: &S,
        events: &EventBus,
    ) -> anyhow::Result<()> {
        let member_ids: Vec<String> = self
            .sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        for member_id in member_ids {
            self.stop_member(store, events, &member_id).await?;
        }
        Ok(())
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}
