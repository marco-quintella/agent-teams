use std::path::{Path, PathBuf};

/// Paths under the project root for a team member workspace.
#[derive(Debug, Clone)]
pub struct MemberWorkspace {
    pub root: PathBuf,
    pub protocol_file: PathBuf,
    pub inbound_file: PathBuf,
    pub role_file: PathBuf,
}

impl MemberWorkspace {
    pub fn new(project_root: &Path, team_id: &str, member_id: &str) -> Self {
        let root = project_root.join(".orchestrator").join("teams").join(team_id).join(member_id);
        Self {
            protocol_file: root.join("protocol.ndjson"),
            inbound_file: root.join("inbound.md"),
            role_file: root.join("role.md"),
            root,
        }
    }

    pub async fn ensure(&self) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        if !self.protocol_file.exists() {
            tokio::fs::write(&self.protocol_file, "").await?;
        }
        Ok(())
    }
}
