use std::path::Path;

use crate::domain::{MemberRole, Task, TeamMember};
use crate::supervisor::workspace::MemberWorkspace;

/// Builds argv for a persistent interactive Claude Code session (no `-p` / `--version`).
pub fn claude_spawn_args(role_file: &Path, project_root: &Path) -> Vec<String> {
    vec![
        "--append-system-prompt-file".to_string(),
        role_file.to_string_lossy().into_owned(),
        "--add-dir".to_string(),
        project_root.to_string_lossy().into_owned(),
    ]
}

pub fn build_role_markdown(
    member: &TeamMember,
    team_prompt: &str,
    atop_spec: &str,
    workspace: &MemberWorkspace,
) -> String {
    let role_label = match member.role {
        MemberRole::Lead => "lead",
        MemberRole::Worker => "worker",
    };

    let protocol_path = workspace.protocol_file.display();
    let inbound_path = workspace.inbound_file.display();

    let lead_atop_directive = if member.role == MemberRole::Lead {
        "\n## Lead protocol duty\n\nWhen the operator sends an `[orchestrator-objective]` message, your **first** action is to append a `task.create` line to the protocol file with a title that reflects the objective. Use Bash to append one JSON line per the ATOP spec. You may follow with `task.assign` to workers when appropriate.\n"
    } else {
        ""
    };

    format!(
        r#"# Role: {name} ({role_label})

## Team objective
{team_prompt}

## Member focus
{role_prompt}

## Workspace paths
- Protocol (append one JSON object per line): `{protocol_path}`
- Inbound audit log: `{inbound_path}`

## ATOP v1
{atop_spec}

Append **one JSON object per line** to the protocol file for every task mutation. Valid ops: `task.create`, `task.update_status`, `task.assign`, `ping`.
{lead_atop_directive}
"#,
        name = member.name,
        role_prompt = member.role_prompt,
    )
}

/// Initial message written to a member PTY right after spawn.
pub fn format_session_bootstrap(workspace: &MemberWorkspace, task_count: usize) -> String {
    format!(
        r#"[orchestrator-bootstrap]
Session ready. Protocol file: {}
Current board tasks: {task_count}.
Awaiting operator objectives (lead) or assigned work (workers).
"#,
        workspace.protocol_file.display(),
        task_count = task_count,
    )
}

/// Operator objective delivered to the lead's live session.
pub fn format_objective_envelope(tasks: &[Task], objective: &str) -> String {
    let mut lines = vec![
        "[orchestrator-objective]".to_string(),
        String::new(),
        "Current kanban tasks:".to_string(),
    ];

    if tasks.is_empty() {
        lines.push("  (none)".to_string());
    } else {
        for t in tasks {
            let assignee = t
                .assignee_member_id
                .as_deref()
                .unwrap_or("unassigned");
            lines.push(format!(
                "  - [{}] {} ({:?}, assignee: {})",
                t.id, t.title, t.status, assignee
            ));
        }
    }

    lines.push(String::new());
    lines.push("Operator objective:".to_string());
    lines.push(objective.to_string());
    lines.push(String::new());
    lines.push(
        "Respond by appending a task.create JSON line to your protocol file (see role.md)."
            .to_string(),
    );

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::domain::{Task, TaskActor, TaskStatus};

    #[test]
    fn claude_spawn_args_exclude_print_and_version() {
        let args = claude_spawn_args(Path::new("/proj/.orchestrator/role.md"), Path::new("/proj"));
        assert!(!args.iter().any(|a| a == "--version"));
        assert!(!args.iter().any(|a| a == "-p" || a == "--print"));
        assert!(args.contains(&"--append-system-prompt-file".to_string()));
        assert!(args.contains(&"--add-dir".to_string()));
    }

    #[test]
    fn objective_envelope_includes_objective_and_tasks() {
        let task = Task {
            id: "t1".into(),
            team_id: "team".into(),
            title: "Fix tests".into(),
            description: String::new(),
            status: TaskStatus::Backlog,
            assignee_member_id: None,
            created_by: TaskActor::Human,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let body = format_objective_envelope(&[task], "Add CONTRIBUTING.md");
        assert!(body.contains("[orchestrator-objective]"));
        assert!(body.contains("Add CONTRIBUTING.md"));
        assert!(body.contains("Fix tests"));
    }

    #[test]
    fn lead_role_markdown_mentions_protocol_path() {
        let ws = MemberWorkspace::new(Path::new("/proj"), "team1", "mem1");
        let member = TeamMember {
            id: "m1".into(),
            team_id: "team1".into(),
            name: "Lead".into(),
            role: MemberRole::Lead,
            role_prompt: "Coordinate".into(),
            created_at: Utc::now(),
        };
        let md = build_role_markdown(&member, "Ship V1.1", "spec", &ws);
        assert!(md.contains("task.create"));
        assert!(md.contains(&ws.protocol_file.display().to_string()));
    }
}
