use serde::Serialize;
use tokio::sync::broadcast;

use crate::domain::{AgentRun, Task};

/// Realtime events pushed to WebSocket subscribers.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestratorEvent {
    TaskUpdated { task: Task },
    AgentRunUpdated { run: AgentRun },
    TeamUpdated { team_id: String },
}

/// In-process broadcast bus for UI updates.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<OrchestratorEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: OrchestratorEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<OrchestratorEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}
