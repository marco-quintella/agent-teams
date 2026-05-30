use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use std::sync::Mutex;

use crate::domain::AgentRunStatus;
use crate::supervisor::workspace::MemberWorkspace;

const SNIPPET_MAX: usize = 2048;

/// Live PTY session for one team member.
pub struct MemberSession {
    pub member_id: String,
    pub workspace: MemberWorkspace,
    pub last_output: Arc<Mutex<String>>,
    stdin_writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    child: Arc<std::sync::Mutex<Option<Box<dyn portable_pty::Child + Send>>>>,
    reader_handle: Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl MemberSession {
    pub fn spawn(
        project_root: &Path,
        team_id: &str,
        member_id: &str,
        command: &Path,
        args: &[String],
        role_markdown: &str,
        extra_env: &[(String, String)],
    ) -> anyhow::Result<Self> {
        let workspace = MemberWorkspace::new(project_root, team_id, member_id);
        std::fs::create_dir_all(&workspace.root)?;
        std::fs::write(&workspace.role_file, role_markdown)?;
        if !workspace.protocol_file.exists() {
            std::fs::File::create(&workspace.protocol_file)?;
        }

        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(command);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.cwd(project_root);
        if cfg!(windows) {
            cmd.env("TERM", "xterm-256color");
        }
        for (k, v) in extra_env {
            cmd.env(k, v);
        }

        let child = pair.slave.spawn_command(cmd)?;
        let writer = pair.master.take_writer()?;
        let mut reader = pair.master.try_clone_reader()?;

        let snippet = Arc::new(Mutex::new(String::new()));
        let snippet_reader = Arc::clone(&snippet);
        let stdin_writer: Arc<Mutex<Option<Box<dyn Write + Send>>>> =
            Arc::new(Mutex::new(Some(writer)));

        let reader_handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        if let Ok(mut guard) = snippet_reader.lock() {
                            guard.push_str(&chunk);
                            if guard.len() > SNIPPET_MAX {
                                let drain = guard.len() - SNIPPET_MAX;
                                guard.drain(..drain);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            member_id: member_id.to_string(),
            workspace,
            last_output: snippet,
            stdin_writer,
            child: Arc::new(std::sync::Mutex::new(Some(child))),
            reader_handle: Arc::new(std::sync::Mutex::new(Some(reader_handle))),
        })
    }

    pub async fn last_output_snippet(&self) -> String {
        self.last_output
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    /// Writes to the live PTY and appends the same text to the inbound audit file.
    pub fn write_stdin(&self, text: &str) -> anyhow::Result<()> {
        let payload = if text.ends_with('\n') {
            text.to_string()
        } else {
            format!("{text}\n")
        };

        {
            let mut guard = self
                .stdin_writer
                .lock()
                .map_err(|_| anyhow::anyhow!("stdin writer lock poisoned"))?;
            let writer = guard
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("PTY stdin writer is closed"))?;
            writer.write_all(payload.as_bytes())?;
            writer.flush()?;
        }

        let mut content = String::new();
        if self.workspace.inbound_file.exists() {
            content = std::fs::read_to_string(&self.workspace.inbound_file)?;
        }
        content.push_str(&payload);
        std::fs::write(&self.workspace.inbound_file, content)?;
        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.stdin_writer.lock() {
            guard.take();
        }
        if let Ok(mut guard) = self.child.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        if let Ok(mut handle) = self.reader_handle.lock() {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        let stdin_open = self
            .stdin_writer
            .lock()
            .map(|g| g.is_some())
            .unwrap_or(false);
        if !stdin_open {
            return false;
        }
        let Ok(mut guard) = self.child.lock() else {
            return false;
        };
        let Some(child) = guard.as_mut() else {
            return false;
        };
        match child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    pub fn status_hint(&self) -> AgentRunStatus {
        if self.is_alive() {
            AgentRunStatus::Running
        } else {
            AgentRunStatus::Stopped
        }
    }
}

impl Drop for MemberSession {
    fn drop(&mut self) {
        self.stop();
    }
}
