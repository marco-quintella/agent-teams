mod session;
mod workspace;

use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;

use crate::agents::ClaudeCodeAgent;
use crate::domain::{AgentRun, AgentRunStatus, MemberRole, TeamMember};
use crate::events::{EventBus, OrchestratorEvent};
use crate::store::{Store, new_agent_run};

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

    fn build_role_markdown(
        member: &TeamMember,
        team_prompt: &str,
        atop_spec: &str,
    ) -> String {
        format!(
            "# Role: {} ({})\n\n## Team objective\n{team_prompt}\n\n## Member focus\n{}\n\n## ATOP\n{atop_spec}\n",
            member.name,
            match member.role {
                MemberRole::Lead => "lead",
                MemberRole::Worker => "worker",
            },
            member.role_prompt,
        )
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
    ) -> anyhow::Result<AgentRun> {
        if self.sessions.contains_key(&member.id) {
            anyhow::bail!("member {} already has a live session", member.id);
        }

        let mut run = new_agent_run(&member.id);
        run.status = AgentRunStatus::Starting;
        store.upsert_agent_run(&run).await?;

        let role_md = Self::build_role_markdown(member, team_prompt, atop_spec);

        let (cmd_path, args): (PathBuf, Vec<String>) = if let Some((path, args)) = mock_command {
            (path.to_path_buf(), args.to_vec())
        } else {
            let path = self
                .claude_path
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Claude CLI not found on PATH"))?;
            (path, vec!["--version".to_string()])
        };

        let session = Arc::new(MemberSession::spawn(
            project_root,
            team_id,
            &member.id,
            &cmd_path,
            &args,
            &role_md,
        )?);

        self.sessions.insert(member.id.clone(), Arc::clone(&session));

        run.status = AgentRunStatus::Running;
        run.pid = None;
        run.last_output_snippet = session.last_output_snippet().await;
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

    /// Appends operator text to the lead member's inbound channel.
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
        session.write_stdin(text)?;
        Ok(())
    }

    pub fn has_live_sessions(&self, member_ids: &[String]) -> bool {
        member_ids.iter().any(|id| self.sessions.contains_key(id))
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}
