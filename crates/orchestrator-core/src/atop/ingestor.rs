use std::path::Path;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::fs::File;
use tokio::io::SeekFrom;

use crate::atop::schema::AtopMessage;
use crate::domain::TaskActor;
use crate::events::{EventBus, OrchestratorEvent};
use crate::store::Store;

/// Tails a member's protocol file and applies task mutations.
pub struct AtopIngestor;

impl AtopIngestor {
    pub async fn process_line<S: Store + ?Sized>(
        store: &S,
        events: &EventBus,
        team_id: &str,
        line: &str,
    ) -> anyhow::Result<()> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        let msg: AtopMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "invalid atop line");
                return Ok(());
            }
        };

        match msg {
            AtopMessage::TaskCreate {
                title,
                description,
                status,
                assignee,
            } => {
                let status = AtopMessage::parse_status(&status).unwrap_or(crate::domain::TaskStatus::Backlog);
                let task = store
                    .create_task(
                        team_id,
                        &title,
                        &description,
                        status,
                        assignee.as_deref(),
                        TaskActor::Agent,
                    )
                    .await?;
                events.publish(OrchestratorEvent::TaskUpdated { task });
            }
            AtopMessage::TaskUpdateStatus { task_id, status } => {
                if let Some(status) = AtopMessage::parse_status(&status) {
                    let task = store
                        .update_task_status(&task_id, status, TaskActor::Agent)
                        .await?;
                    events.publish(OrchestratorEvent::TaskUpdated { task });
                }
            }
            AtopMessage::TaskAssign { task_id, assignee } => {
                let task = store
                    .assign_task(&task_id, assignee.as_deref(), TaskActor::Agent)
                    .await?;
                events.publish(OrchestratorEvent::TaskUpdated { task });
            }
            AtopMessage::Ping => {}
        }
        Ok(())
    }

    /// Polls the protocol file for new lines (simple tail loop for V1).
    pub async fn watch_file<S: Store + ?Sized>(
        store: &S,
        events: &EventBus,
        team_id: &str,
        path: &Path,
        mut offset: u64,
    ) -> anyhow::Result<u64> {
        let mut file = File::open(path).await?;
        file.seek(SeekFrom::Start(offset)).await?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                break;
            }
            offset += n as u64;
            Self::process_line(store, events, team_id, &line).await?;
        }
        Ok(offset)
    }

    pub async fn run_loop<S: Store + ?Sized>(
        store: &S,
        events: &EventBus,
        team_id: &str,
        path: std::path::PathBuf,
    ) {
        let mut offset = 0u64;
        loop {
            if path.exists() {
                match Self::watch_file(store, events, team_id, &path, offset).await {
                    Ok(new_offset) => offset = new_offset,
                    Err(e) => tracing::warn!(error = %e, "atop tail error"),
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
