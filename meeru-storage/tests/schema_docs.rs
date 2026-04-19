use std::{collections::BTreeSet, path::PathBuf};

use meeru_storage::migrations;
use sqlx::{raw_sql, sqlite::SqliteConnectOptions, SqlitePool};
use tempfile::TempDir;

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
    let expected: BTreeSet<String> = migrations::V1_OBJECT_NAMES
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
        "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, description TEXT);\n",
    );
    for block in sql_blocks {
        all_sql.push_str(&block);
        all_sql.push('\n');
    }
    all_sql
}

async fn object_names(pool: &SqlitePool) -> BTreeSet<String> {
    let placeholders = std::iter::repeat_n("?", migrations::V1_OBJECT_NAMES.len())
        .collect::<Vec<_>>()
        .join(", ");
    let query =
        format!("SELECT name FROM sqlite_master WHERE name IN ({placeholders}) ORDER BY name");

    let mut sql = sqlx::query(&query);
    for name in migrations::V1_OBJECT_NAMES {
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
