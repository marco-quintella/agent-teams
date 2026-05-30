use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use orchestrator_core::{
    cli_login_marker_present, credentials_ready, decrypt_settings_api_key, encrypt_api_key,
    load_or_create_master_key, mask_api_key, ClaudeCodeAgent, ClaudeSettings, ClaudeSettingsView,
    CredentialMode, Store,
};
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/setup/doctor", get(doctor))
        .route(
            "/setup/claude-settings",
            get(get_claude_settings).patch(patch_claude_settings),
        )
        .route("/setup/claude-login", post(claude_login))
        .route("/setup/install-claude", post(install_claude))
}

#[derive(Serialize)]
struct DoctorResponse {
    orchestrator_version: String,
    cli: CliDoctor,
    credentials: CredentialsDoctor,
}

#[derive(Serialize)]
struct CliDoctor {
    found: bool,
    version: Option<String>,
}

#[derive(Serialize)]
struct CredentialsDoctor {
    mode: String,
    ready: bool,
    hint: String,
}

async fn doctor(State(state): State<AppState>) -> Json<DoctorResponse> {
    let settings = state.store.get_claude_settings().await.unwrap_or(ClaudeSettings {
        credential_mode: CredentialMode::CliLogin,
        api_key_ciphertext: None,
        api_base_url: None,
        updated_at: Utc::now(),
    });

    let cli_found = ClaudeCodeAgent::is_available();
    let version = if cli_found {
        ClaudeCodeAgent::new()
            .ok()
            .and_then(|a| {
                Command::new(a.executable_path())
                    .arg("--version")
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            })
            .filter(|s| !s.is_empty())
    } else {
        None
    };

    let cred_ready = credentials_ready(&settings, &state.config.data_dir).unwrap_or(false);
    let hint = credential_hint(&settings, cli_found, cred_ready);

    Json(DoctorResponse {
        orchestrator_version: env!("CARGO_PKG_VERSION").into(),
        cli: CliDoctor {
            found: cli_found,
            version,
        },
        credentials: CredentialsDoctor {
            mode: settings.credential_mode.as_str().into(),
            ready: cred_ready,
            hint,
        },
    })
}

fn credential_hint(settings: &ClaudeSettings, cli_found: bool, cred_ready: bool) -> String {
    if !cli_found {
        return "Install Claude Code CLI (Settings → Install) or add claude to PATH.".into();
    }
    if cred_ready {
        return "Ready to launch.".into();
    }
    match settings.credential_mode {
        CredentialMode::CliLogin => {
            "Run CLI login from Settings or complete claude login in a terminal.".into()
        }
        CredentialMode::ApiKey => "Save an API key in Settings (api_key mode).".into(),
    }
}

async fn get_claude_settings(State(state): State<AppState>) -> Json<ClaudeSettingsView> {
    let settings = state
        .store
        .get_claude_settings()
        .await
        .unwrap_or(ClaudeSettings {
            credential_mode: CredentialMode::CliLogin,
            api_key_ciphertext: None,
            api_base_url: None,
            updated_at: Utc::now(),
        });
    Json(settings_to_view(&settings, &state.config.data_dir))
}

#[derive(Deserialize)]
struct PatchClaudeSettings {
    credential_mode: String,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    api_base_url: Option<String>,
}

async fn patch_claude_settings(
    State(state): State<AppState>,
    Json(body): Json<PatchClaudeSettings>,
) -> Result<Json<ClaudeSettingsView>, (StatusCode, String)> {
    let mode = CredentialMode::parse(&body.credential_mode).ok_or((
        StatusCode::BAD_REQUEST,
        "credential_mode must be cli_login or api_key".into(),
    ))?;

    let mut settings = state.store.get_claude_settings().await.map_err(internal)?;

    if mode == CredentialMode::ApiKey {
        if let Some(key) = body
            .api_key
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            let master = load_or_create_master_key(&state.config.data_dir).map_err(internal)?;
            settings.api_key_ciphertext =
                Some(encrypt_api_key(key, &master).map_err(internal)?);
        } else if settings.api_key_ciphertext.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                "api_key required when switching to api_key mode".into(),
            ));
        }
    } else {
        settings.api_key_ciphertext = None;
    }

    settings.credential_mode = mode;
    settings.api_base_url = body
        .api_base_url
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    settings.updated_at = Utc::now();

    state
        .store
        .upsert_claude_settings(&settings)
        .await
        .map_err(internal)?;

    Ok(Json(settings_to_view(&settings, &state.config.data_dir)))
}

fn settings_to_view(settings: &ClaudeSettings, data_dir: &std::path::Path) -> ClaudeSettingsView {
    let api_key_masked = if settings.credential_mode == CredentialMode::ApiKey {
        decrypt_settings_api_key(settings, data_dir)
            .ok()
            .flatten()
            .map(|k| mask_api_key(&k))
    } else {
        None
    };
    ClaudeSettingsView {
        credential_mode: settings.credential_mode.as_str().into(),
        api_key_masked,
        api_base_url: settings.api_base_url.clone(),
        updated_at: settings.updated_at.to_rfc3339(),
    }
}

#[derive(Serialize)]
struct ClaudeLoginResponse {
    ok: bool,
    message: String,
}

async fn claude_login(State(state): State<AppState>) -> Result<Json<ClaudeLoginResponse>, (StatusCode, String)> {
    let agent = ClaudeCodeAgent::new().map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "claude CLI not found on PATH".into(),
        )
    })?;

    let output = Command::new(agent.executable_path())
        .arg("login")
        .output()
        .map_err(|e| internal(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}").trim().to_string();

    let mut settings = state.store.get_claude_settings().await.map_err(internal)?;
    settings.credential_mode = CredentialMode::CliLogin;
    settings.api_key_ciphertext = None;
    settings.updated_at = Utc::now();
    state
        .store
        .upsert_claude_settings(&settings)
        .await
        .map_err(internal)?;

    let ready = cli_login_marker_present() || output.status.success();
    Ok(Json(ClaudeLoginResponse {
        ok: ready,
        message: if combined.is_empty() {
            if ready {
                "CLI login completed.".into()
            } else {
                "CLI login finished; verify credentials in Settings doctor.".into()
            }
        } else {
            combined
        },
    }))
}

#[derive(Deserialize)]
struct InstallClaudeBody {
    confirm: bool,
}

#[derive(Serialize)]
struct InstallClaudeResponse {
    ok: bool,
    command: String,
    output: String,
}

async fn install_claude(
    Json(body): Json<InstallClaudeBody>,
) -> Result<Json<InstallClaudeResponse>, (StatusCode, String)> {
    if !body.confirm {
        return Err((
            StatusCode::BAD_REQUEST,
            "confirm: true required to install Claude CLI".into(),
        ));
    }

    #[cfg(windows)]
    let (cmd, args): (&str, Vec<&str>) = (
        "winget",
        vec![
            "install",
            "-e",
            "--id",
            "Anthropic.ClaudeCode",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    );

    #[cfg(not(windows))]
    let (cmd, args): (&str, Vec<&str>) = (
        "sh",
        vec!["-c", "curl -fsSL https://claude.ai/install.sh | bash"],
    );

    let output = Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| internal(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = format!("{stdout}{stderr}");

    Ok(Json(InstallClaudeResponse {
        ok: output.status.success() || ClaudeCodeAgent::is_available(),
        command: format!("{cmd} {}", args.join(" ")),
        output: text,
    }))
}

fn internal<E: ToString>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
