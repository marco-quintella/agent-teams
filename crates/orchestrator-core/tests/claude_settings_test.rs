use orchestrator_core::{
    encrypt_api_key, load_or_create_master_key, CredentialMode, SqliteStore, Store,
};
use tempfile::tempdir;

#[tokio::test]
async fn claude_settings_roundtrip() {
    let dir = tempdir().unwrap();
    let db_url = format!("sqlite:{}", dir.path().join("t.db").display());
    let store = SqliteStore::connect(&db_url).await.unwrap();

    let master = load_or_create_master_key(dir.path()).unwrap();
    let ct = encrypt_api_key("sk-openrouter-test", &master).unwrap();

    let mut settings = store.get_claude_settings().await.unwrap();
    settings.credential_mode = CredentialMode::ApiKey;
    settings.api_key_ciphertext = Some(ct);
    settings.api_base_url = Some("https://openrouter.ai/api".into());
    store.upsert_claude_settings(&settings).await.unwrap();

    let loaded = store.get_claude_settings().await.unwrap();
    assert_eq!(loaded.credential_mode, CredentialMode::ApiKey);
    assert!(loaded.api_key_ciphertext.is_some());
    assert_eq!(
        loaded.api_base_url.as_deref(),
        Some("https://openrouter.ai/api")
    );
}
