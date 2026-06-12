use anyhow::{Context as _, Result};
use rusqlite::Connection;
use std::path::Path;

pub fn open(path: &str) -> Result<Connection> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).context("failed to create database directory")?;
        }
    }
    let conn = Connection::open(path).context("failed to open database")?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .context("failed to set PRAGMAs")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'free',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            key_hash TEXT NOT NULL,
            prefix TEXT NOT NULL,
            last_four TEXT NOT NULL,
            environment TEXT NOT NULL DEFAULT 'live',
            status TEXT NOT NULL DEFAULT 'active',
            lineage_id TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            expires_at TEXT,
            last_used_at TEXT,
            revoked_at TEXT
        );

        CREATE TABLE IF NOT EXISTS key_scopes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key_id TEXT NOT NULL REFERENCES api_keys(id),
            service TEXT NOT NULL,
            permission TEXT NOT NULL,
            UNIQUE(key_id, service, permission)
        );

        CREATE TABLE IF NOT EXISTS usage_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            method TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            response_time_ms INTEGER NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS webhook_configs (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL REFERENCES users(id),
            url TEXT NOT NULL,
            secret_hash TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS webhook_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            webhook_id TEXT NOT NULL REFERENCES webhook_configs(id),
            event_type TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS webhook_deliveries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            webhook_id TEXT NOT NULL REFERENCES webhook_configs(id),
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            status_code INTEGER,
            attempt INTEGER NOT NULL DEFAULT 1,
            success INTEGER NOT NULL DEFAULT 0,
            error_message TEXT,
            delivered_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
        CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(prefix);
        CREATE INDEX IF NOT EXISTS idx_api_keys_lineage_id ON api_keys(lineage_id);
        CREATE INDEX IF NOT EXISTS idx_key_scopes_key_id ON key_scopes(key_id);
        CREATE INDEX IF NOT EXISTS idx_usage_log_key_id ON usage_log(key_id);
        CREATE INDEX IF NOT EXISTS idx_usage_log_timestamp ON usage_log(timestamp);
        CREATE INDEX IF NOT EXISTS idx_webhook_configs_user_id ON webhook_configs(user_id);
    ",
    )
    .context("migration failed")
}
