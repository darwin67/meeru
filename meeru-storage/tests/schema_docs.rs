use std::{collections::BTreeSet, path::PathBuf};

use meeru_storage::migrations;
use sqlx::{raw_sql, sqlite::SqliteConnectOptions, SqlitePool};
use tempfile::TempDir;

const V1_OBJECT_NAMES: &[&str] = &[
    "migrations",
    "accounts",
    "idx_accounts_email",
    "idx_accounts_active",
    "unified_folders",
    "idx_unified_folders_parent",
    "idx_unified_folders_type",
    "folder_mappings",
    "idx_folder_mappings_unified",
    "idx_folder_mappings_account",
    "emails",
    "idx_emails_account",
    "idx_emails_thread",
    "idx_emails_message_id",
    "idx_emails_from",
    "idx_emails_date",
    "idx_emails_unread",
    "idx_emails_starred",
    "idx_emails_search",
    "email_folders",
    "idx_email_folders_folder",
    "attachments",
    "idx_attachments_email",
];

const V1_HEADINGS: &[&str] = &[
    "Accounts Table",
    "Unified Folders Table",
    "Folder Mappings Table",
    "Emails Table",
    "Email Folder Associations",
    "Attachments Table",
];

#[tokio::test]
async fn documented_v1_schema_executes_in_sqlite() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("open in-memory sqlite");

    raw_sql(&documented_v1_sql())
        .execute(&pool)
        .await
        .expect("execute documented v1 schema");

    let names = object_names(&pool).await;
    let expected: BTreeSet<String> = V1_OBJECT_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect();

    assert_eq!(names, expected);
}

#[tokio::test]
async fn documented_v1_schema_matches_migration_objects() {
    let docs_pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("open docs sqlite");
    raw_sql(&documented_v1_sql())
        .execute(&docs_pool)
        .await
        .expect("execute documented schema");

    let temp_dir = TempDir::new().expect("temp dir");
    let migration_pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(temp_dir.path().join("migration.db"))
            .create_if_missing(true),
    )
    .await
    .expect("open migration sqlite");
    migrations::run_migrations(&migration_pool)
        .await
        .expect("run migrations");

    assert_eq!(
        object_names(&docs_pool).await,
        object_names(&migration_pool).await
    );
}

fn documented_v1_sql() -> String {
    let doc = std::fs::read_to_string(database_schema_doc_path()).expect("read schema doc");
    let mut sql_blocks = Vec::new();
    let mut active_heading: Option<&str> = None;
    let mut current_block = String::new();
    let mut in_sql_block = false;

    for line in doc.lines() {
        if let Some(heading) = line.strip_prefix("** ") {
            active_heading = V1_HEADINGS
                .iter()
                .copied()
                .find(|candidate| *candidate == heading);
            continue;
        }

        if active_heading.is_some() && line == "#+begin_src sql" {
            in_sql_block = true;
            current_block.clear();
            continue;
        }

        if in_sql_block && line == "#+end_src" {
            in_sql_block = false;
            sql_blocks.push(current_block.trim().to_string());
            current_block.clear();
            continue;
        }

        if in_sql_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    let mut all_sql = String::from(
        "CREATE TABLE migrations (id INTEGER PRIMARY KEY AUTOINCREMENT, version_id INTEGER NOT NULL, is_applied INTEGER NOT NULL, tstamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP);\n",
    );
    for block in sql_blocks {
        all_sql.push_str(&block);
        all_sql.push('\n');
    }
    all_sql
}

async fn object_names(pool: &SqlitePool) -> BTreeSet<String> {
    let placeholders = std::iter::repeat_n("?", V1_OBJECT_NAMES.len())
        .collect::<Vec<_>>()
        .join(", ");
    let query =
        format!("SELECT name FROM sqlite_master WHERE name IN ({placeholders}) ORDER BY name");

    let mut sql = sqlx::query(&query);
    for name in V1_OBJECT_NAMES {
        sql = sql.bind(name);
    }

    sql.fetch_all(pool)
        .await
        .expect("query sqlite_master")
        .into_iter()
        .map(|row| sqlx::Row::get::<String, _>(&row, "name"))
        .collect()
}

fn database_schema_doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("database-schema.org")
}
