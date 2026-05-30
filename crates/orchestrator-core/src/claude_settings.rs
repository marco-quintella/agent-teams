use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SETTINGS_ROW_ID: &str = "default";
const KEY_FILE: &str = "orchestrator.key";
const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialMode {
    CliLogin,
    ApiKey,
}

impl CredentialMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CliLogin => "cli_login",
            Self::ApiKey => "api_key",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cli_login" => Some(Self::CliLogin),
            "api_key" => Some(Self::ApiKey),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClaudeSettings {
    pub credential_mode: CredentialMode,
    pub api_key_ciphertext: Option<Vec<u8>>,
    pub api_base_url: Option<String>,
    pub default_model: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeSettingsView {
    pub credential_mode: String,
    pub api_key_masked: Option<String>,
    pub api_base_url: Option<String>,
    pub default_model: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LaunchEnv {
    pub vars: std::collections::HashMap<String, String>,
}

impl LaunchEnv {
    pub fn from_api_key(api_key: &str, api_base_url: Option<&str>) -> Self {
        let mut vars = std::collections::HashMap::new();
        vars.insert("ANTHROPIC_API_KEY".into(), api_key.to_string());
        if let Some(url) = api_base_url.filter(|s| !s.trim().is_empty()) {
            vars.insert("ANTHROPIC_BASE_URL".into(), url.to_string());
        }
        Self { vars }
    }
}

pub fn mask_api_key(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.len() <= 8 {
        return "****".to_string();
    }
    let last4 = &trimmed[trimmed.len() - 4..];
    format!("sk-…{last4}")
}

pub fn settings_row_id() -> &'static str {
    SETTINGS_ROW_ID
}

/// Loads or creates a 32-byte master key under `data_dir`.
pub fn load_or_create_master_key(data_dir: &Path) -> anyhow::Result<[u8; 32]> {
    std::fs::create_dir_all(data_dir)?;
    let path = data_dir.join(KEY_FILE);
    if path.exists() {
        let bytes = std::fs::read(&path)?;
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
        anyhow::bail!("invalid master key file at {}", path.display());
    }
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    std::fs::write(&path, key)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }
    Ok(key)
}

pub fn encrypt_api_key(plaintext: &str, master_key: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(master_key)
        .map_err(|e| anyhow::anyhow!("cipher init: {e}"))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("encrypt failed: {e}"))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_api_key(blob: &[u8], master_key: &[u8; 32]) -> anyhow::Result<String> {
    if blob.len() < NONCE_LEN + 16 {
        anyhow::bail!("ciphertext too short");
    }
    let (nonce_bytes, ct) = blob.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new_from_slice(master_key)
        .map_err(|e| anyhow::anyhow!("cipher init: {e}"))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ct)
        .map_err(|_| anyhow::anyhow!("decrypt failed (corrupt ciphertext or wrong key)"))?;
    Ok(String::from_utf8(plain)?)
}

/// Returns true when Claude Code credential files exist under the user home.
pub fn cli_login_marker_present() -> bool {
    let home = match std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        Some(h) => PathBuf::from(h),
        None => return false,
    };
    let claude_dir = home.join(".claude");
    if !claude_dir.is_dir() {
        return false;
    }
    claude_dir.join(".credentials.json").exists()
        || claude_dir.join("credentials.json").exists()
        || claude_dir.join("settings.json").exists()
}

pub fn credentials_ready(settings: &ClaudeSettings, data_dir: &Path) -> anyhow::Result<bool> {
    match settings.credential_mode {
        CredentialMode::CliLogin => Ok(cli_login_marker_present()),
        CredentialMode::ApiKey => {
            let Some(blob) = settings.api_key_ciphertext.as_ref() else {
                return Ok(false);
            };
            let key = load_or_create_master_key(data_dir)?;
            let plain = decrypt_api_key(blob, &key)?;
            Ok(!plain.trim().is_empty())
        }
    }
}

pub fn decrypt_settings_api_key(
    settings: &ClaudeSettings,
    data_dir: &Path,
) -> anyhow::Result<Option<String>> {
    if settings.credential_mode != CredentialMode::ApiKey {
        return Ok(None);
    }
    let blob = settings
        .api_key_ciphertext
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("api_key mode without stored ciphertext"))?;
    let key = load_or_create_master_key(data_dir)?;
    Ok(Some(decrypt_api_key(blob, &key)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn encrypt_roundtrip() {
        let dir = tempdir().unwrap();
        let master = load_or_create_master_key(dir.path()).unwrap();
        let ct = encrypt_api_key("sk-test-secret-key", &master).unwrap();
        let plain = decrypt_api_key(&ct, &master).unwrap();
        assert_eq!(plain, "sk-test-secret-key");
    }

    #[test]
    fn mask_hides_middle() {
        let m = mask_api_key("sk-ant-api03-abcdefghijklmnop");
        assert!(m.contains("…"));
        assert!(m.ends_with("mnop"));
    }
}
