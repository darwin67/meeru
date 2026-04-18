//! SQLite schema migrations for the Meeru storage layer.

use sqlx::{raw_sql, Row, SqlitePool};

use crate::Result;

pub const CURRENT_SCHEMA_VERSION: i64 = 1;

struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "initial storage schema",
    sql: r#"
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    provider_type TEXT NOT NULL CHECK (provider_type IN ('gmail', 'outlook', 'generic')),
    imap_host TEXT,
    imap_port INTEGER,
    imap_security TEXT CHECK (imap_security IN ('tls', 'starttls', 'none')),
    smtp_host TEXT,
    smtp_port INTEGER,
    smtp_security TEXT CHECK (smtp_security IN ('tls', 'starttls', 'none')),
    auth_method TEXT CHECK (auth_method IN ('password', 'oauth2')),
    encrypted_credentials TEXT,
    oauth_refresh_token TEXT,
    oauth_access_token TEXT,
    oauth_token_expires TIMESTAMP,
    sync_enabled INTEGER NOT NULL DEFAULT 1,
    sync_interval_minutes INTEGER NOT NULL DEFAULT 15,
    last_sync_time TIMESTAMP,
    last_sync_status TEXT,
    provider_settings TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts(email);
CREATE INDEX IF NOT EXISTS idx_accounts_active ON accounts(is_active);

CREATE TABLE IF NOT EXISTS unified_folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    folder_type TEXT NOT NULL,
    parent_id TEXT REFERENCES unified_folders(id),
    icon TEXT,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_smart_folder INTEGER NOT NULL DEFAULT 0,
    smart_rules TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_unified_folders_parent ON unified_folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_unified_folders_type ON unified_folders(folder_type);

CREATE TABLE IF NOT EXISTS folder_mappings (
    id TEXT PRIMARY KEY,
    unified_folder_id TEXT NOT NULL REFERENCES unified_folders(id),
    account_id TEXT NOT NULL REFERENCES accounts(id),
    provider_folder_id TEXT NOT NULL,
    provider_folder_name TEXT,
    mapping_type TEXT CHECK (mapping_type IN ('direct', 'label', 'virtual', 'computed')),
    sync_direction TEXT CHECK (sync_direction IN ('bidirectional', 'to_unified', 'to_provider')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(account_id, provider_folder_id)
);

CREATE INDEX IF NOT EXISTS idx_folder_mappings_unified ON folder_mappings(unified_folder_id);
CREATE INDEX IF NOT EXISTS idx_folder_mappings_account ON folder_mappings(account_id);

CREATE TABLE IF NOT EXISTS emails (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    provider_id TEXT NOT NULL,
    message_id TEXT,
    thread_id TEXT,
    subject TEXT,
    from_address TEXT,
    from_name TEXT,
    to_addresses TEXT,
    cc_addresses TEXT,
    bcc_addresses TEXT,
    reply_to TEXT,
    date_sent TIMESTAMP,
    date_received TIMESTAMP,
    date_internal TIMESTAMP,
    content_file_path TEXT,
    content_hash TEXT,
    size_bytes INTEGER,
    has_attachments INTEGER NOT NULL DEFAULT 0,
    attachment_count INTEGER NOT NULL DEFAULT 0,
    is_read INTEGER NOT NULL DEFAULT 0,
    is_starred INTEGER NOT NULL DEFAULT 0,
    is_important INTEGER NOT NULL DEFAULT 0,
    is_draft INTEGER NOT NULL DEFAULT 0,
    is_sent INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    ai_category TEXT,
    ai_priority INTEGER,
    ai_summary TEXT,
    ai_sentiment TEXT,
    ai_processed_at TIMESTAMP,
    search_text TEXT,
    tantivy_doc_id TEXT,
    sync_state TEXT CHECK (sync_state IN ('synced', 'pending_local', 'pending_remote', 'conflict')),
    last_modified TIMESTAMP,
    provider_labels TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(account_id, provider_id)
);

CREATE INDEX IF NOT EXISTS idx_emails_account ON emails(account_id);
CREATE INDEX IF NOT EXISTS idx_emails_thread ON emails(thread_id);
CREATE INDEX IF NOT EXISTS idx_emails_message_id ON emails(message_id);
CREATE INDEX IF NOT EXISTS idx_emails_from ON emails(from_address);
CREATE INDEX IF NOT EXISTS idx_emails_date ON emails(date_internal DESC);
CREATE INDEX IF NOT EXISTS idx_emails_unread ON emails(is_read, account_id);
CREATE INDEX IF NOT EXISTS idx_emails_starred ON emails(is_starred, account_id);
CREATE INDEX IF NOT EXISTS idx_emails_search ON emails(subject, from_address, from_name);

CREATE TABLE IF NOT EXISTS email_folders (
    email_id TEXT NOT NULL REFERENCES emails(id),
    unified_folder_id TEXT NOT NULL REFERENCES unified_folders(id),
    PRIMARY KEY (email_id, unified_folder_id)
);

CREATE INDEX IF NOT EXISTS idx_email_folders_folder ON email_folders(unified_folder_id);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    email_id TEXT NOT NULL REFERENCES emails(id),
    filename TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER,
    content_id TEXT,
    content_disposition TEXT,
    file_path TEXT,
    file_hash TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_attachments_email ON attachments(email_id);
"#,
}];

/// Apply all known migrations to the provided SQLite pool.
pub async fn run_migrations(pool: &SqlitePool) -> Result<Vec<i64>> {
    ensure_schema_migrations_table(pool).await?;

    let mut applied = Vec::new();
    for migration in MIGRATIONS {
        if is_applied(pool, migration.version).await? {
            continue;
        }

        let mut tx = pool.begin().await?;
        raw_sql(migration.sql).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO schema_migrations (version, description) VALUES (?, ?)")
            .bind(migration.version)
            .bind(migration.description)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        applied.push(migration.version);
    }

    Ok(applied)
}

/// Return the versions already applied to the given database.
pub async fn applied_versions(pool: &SqlitePool) -> Result<Vec<i64>> {
    ensure_schema_migrations_table(pool).await?;

    let rows = sqlx::query("SELECT version FROM schema_migrations ORDER BY version")
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<i64, _>("version"))
        .collect())
}

async fn ensure_schema_migrations_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description TEXT NOT NULL
)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn is_applied(pool: &SqlitePool, version: i64) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = ?")
        .bind(version)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}
