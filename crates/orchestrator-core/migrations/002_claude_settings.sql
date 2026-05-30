CREATE TABLE claude_settings (
    id TEXT PRIMARY KEY NOT NULL,
    credential_mode TEXT NOT NULL,
    api_key_ciphertext BLOB,
    api_base_url TEXT,
    updated_at TEXT NOT NULL
);

INSERT INTO claude_settings (id, credential_mode, updated_at)
VALUES ('default', 'cli_login', '1970-01-01T00:00:00+00:00');
