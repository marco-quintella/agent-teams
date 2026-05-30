/// Expands a leading `~` or `~/…` using the process home directory.
pub fn expand_tilde_path(path: &str) -> String {
    let path = path.trim();
    let home = home_dir();
    if path == "~" {
        return home.unwrap_or_else(|| path.to_string());
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_prefix() {
        if let Ok(home) = std::env::var("HOME") {
            assert_eq!(
                expand_tilde_path("~/code/agent-teams"),
                format!("{home}/code/agent-teams")
            );
        }
    }
}

fn home_dir() -> Option<String> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok()
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok()
    }
}
