//! SQLite schema migrations for the Meeru storage layer.

use include_dir::{include_dir, Dir};
use sqlx::{raw_sql, Row, SqlitePool};

use crate::{Error, Result};

static MIGRATIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

struct Migration {
    version: i64,
    description: String,
    up_sql: String,
    #[allow(dead_code)]
    down_sql: Option<String>,
}

pub fn current_schema_version() -> i64 {
    load_migrations()
        .last()
        .map(|migration| migration.version)
        .unwrap_or(0)
}

/// Apply all known migrations to the provided SQLite pool.
pub async fn run_migrations(pool: &SqlitePool) -> Result<Vec<i64>> {
    ensure_migrations_table(pool).await?;

    let mut applied = Vec::new();
    for migration in load_migrations() {
        if is_applied(pool, migration.version).await? {
            continue;
        }

        let mut tx = pool.begin().await?;
        raw_sql(&migration.up_sql)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                crate::Error::Migration(format!(
                    "failed to apply migration {} ({}): {error}",
                    migration.version, migration.description
                ))
            })?;
        sqlx::query("INSERT INTO migrations (version_id, is_applied) VALUES (?, 1)")
            .bind(migration.version)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                crate::Error::Migration(format!(
                    "failed to record migration {} ({}): {error}",
                    migration.version, migration.description
                ))
            })?;
        tx.commit().await?;

        applied.push(migration.version);
    }

    Ok(applied)
}

fn load_migrations() -> Vec<Migration> {
    let mut migrations = MIGRATIONS_DIR
        .files()
        .map(|file| {
            let file_name = file
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .expect("migration file name should be valid UTF-8");
            let (version, description) = parse_migration_file_name(file_name);

            parse_migration_sql(
                file.contents_utf8()
                    .expect("migration file contents should be valid UTF-8"),
                version,
                &description,
            )
            .expect("migration file contents should be valid")
        })
        .collect::<Vec<_>>();

    migrations.sort_by_key(|migration| migration.version);
    migrations
}

fn parse_migration_file_name(file_name: &str) -> (i64, String) {
    let stem = file_name
        .strip_suffix(".sql")
        .expect("migration files must use the .sql extension");
    let (version, description) = stem
        .split_once('_')
        .expect("migration file name must be formatted as YYYYMMDDHHMMSS_description.sql");

    assert!(
        version.len() == 14 && version.chars().all(|ch| ch.is_ascii_digit()),
        "migration file prefix must use second-precision datetime format YYYYMMDDHHMMSS"
    );

    (
        version
            .parse()
            .expect("migration datetime prefix must parse as an integer"),
        description.replace('_', " "),
    )
}

fn parse_migration_sql(sql: &str, version: i64, description: &str) -> Result<Migration> {
    let mut up_lines = Vec::new();
    let mut down_lines = Vec::new();
    let mut current_section: Option<&str> = None;

    for line in sql.lines() {
        match line.trim() {
            "-- +up" => {
                if current_section.is_some() {
                    return Err(Error::Migration(format!(
                        "migration {version} ({description}) contains multiple section headers before finishing the previous section"
                    )));
                }
                current_section = Some("up");
                continue;
            },
            "-- +down" => {
                if current_section != Some("up") {
                    return Err(Error::Migration(format!(
                        "migration {version} ({description}) must declare +up before +down"
                    )));
                }
                current_section = Some("down");
                continue;
            },
            _ => {},
        }

        match current_section {
            Some("up") => up_lines.push(line),
            Some("down") => down_lines.push(line),
            Some(_) => unreachable!(),
            None if line.trim().is_empty() => {},
            None => {
                return Err(Error::Migration(format!(
                    "migration {version} ({description}) must start with a -- +up section"
                )))
            },
        }
    }

    let up_sql = up_lines.join("\n").trim().to_string();
    if up_sql.is_empty() {
        return Err(Error::Migration(format!(
            "migration {version} ({description}) has an empty +up section"
        )));
    }

    let down_sql = down_lines.join("\n").trim().to_string();

    Ok(Migration {
        version,
        description: description.to_string(),
        up_sql,
        down_sql: (!down_sql.is_empty()).then_some(down_sql),
    })
}

/// Return the versions already applied to the given database.
pub async fn applied_versions(pool: &SqlitePool) -> Result<Vec<i64>> {
    ensure_migrations_table(pool).await?;

    let rows = sqlx::query(
        "SELECT version_id FROM migrations WHERE is_applied = 1 ORDER BY version_id ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<i64, _>("version_id"))
        .collect())
}

async fn ensure_migrations_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version_id INTEGER NOT NULL,
    is_applied INTEGER NOT NULL,
    tstamp TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP
)
        "#,
    )
    .execute(pool)
    .await?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM migrations")
        .fetch_one(pool)
        .await?;
    if count == 0 {
        sqlx::query("INSERT INTO migrations (version_id, is_applied) VALUES (0, 1)")
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn is_applied(pool: &SqlitePool, version: i64) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM migrations WHERE version_id = ? AND is_applied = 1",
    )
    .bind(version)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}
