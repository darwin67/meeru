//! SQLite schema migrations for the Meeru storage layer.

use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    pub version: i64,
    pub description: String,
    pub applied: bool,
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

/// Roll back the latest applied migrations, newest first.
pub async fn rollback_migrations(pool: &SqlitePool, steps: usize) -> Result<Vec<i64>> {
    ensure_migrations_table(pool).await?;

    if steps == 0 {
        return Ok(Vec::new());
    }

    let applied = applied_versions_desc(pool).await?;
    let mut rolled_back = Vec::new();

    for version in applied.into_iter().take(steps) {
        let migration = load_migrations()
            .into_iter()
            .find(|migration| migration.version == version)
            .ok_or_else(|| {
                Error::Migration(format!("no migration file found for version {version}"))
            })?;
        let down_sql = migration.down_sql.as_ref().ok_or_else(|| {
            Error::Migration(format!(
                "migration {} ({}) has no +down section",
                migration.version, migration.description
            ))
        })?;

        let mut tx = pool.begin().await?;
        raw_sql(down_sql).execute(&mut *tx).await.map_err(|error| {
            Error::Migration(format!(
                "failed to roll back migration {} ({}): {error}",
                migration.version, migration.description
            ))
        })?;
        sqlx::query("INSERT INTO migrations (version_id, is_applied) VALUES (?, 0)")
            .bind(migration.version)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                Error::Migration(format!(
                    "failed to record rollback for migration {} ({}): {error}",
                    migration.version, migration.description
                ))
            })?;
        tx.commit().await?;

        rolled_back.push(migration.version);
    }

    Ok(rolled_back)
}

/// Dump the current schema from sqlite_master in dependency-friendly order.
pub async fn dump_schema(pool: &SqlitePool) -> Result<String> {
    let rows = sqlx::query(
        r#"
SELECT type, name, sql
FROM sqlite_master
WHERE sql IS NOT NULL
  AND name NOT LIKE 'sqlite_%'
ORDER BY
  CASE type
    WHEN 'table' THEN 0
    WHEN 'index' THEN 1
    WHEN 'trigger' THEN 2
    ELSE 3
  END,
  name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut output = String::from(
        "-- Generated from the executable meeru-storage SQLite migrations.\n\
-- Do not edit by hand; regenerate with `scripts/migrations.sh dump`.\n\n",
    );

    for row in rows {
        let object_type: String = row.get("type");
        let name: String = row.get("name");
        let sql: String = row.get("sql");

        output.push_str(&format!("-- {object_type}: {name}\n"));
        output.push_str(sql.trim());
        output.push_str(";\n\n");
    }

    Ok(output)
}

/// List all known migration files and whether each is currently applied.
pub async fn list_migrations(pool: &SqlitePool) -> Result<Vec<MigrationStatus>> {
    ensure_migrations_table(pool).await?;

    let states = latest_applied_states(pool).await?;
    Ok(load_migrations()
        .into_iter()
        .map(|migration| MigrationStatus {
            version: migration.version,
            description: migration.description,
            applied: states.get(&migration.version).copied().unwrap_or(false),
        })
        .collect())
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

    let mut versions = applied_versions_desc(pool).await?;
    versions.reverse();
    versions.insert(0, 0);
    Ok(versions)
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
    let state: Option<i64> = sqlx::query_scalar(
        r#"
SELECT is_applied
FROM migrations
WHERE version_id = ?
ORDER BY id DESC
LIMIT 1
        "#,
    )
    .bind(version)
    .fetch_optional(pool)
    .await?;

    Ok(matches!(state, Some(1)))
}

async fn applied_versions_desc(pool: &SqlitePool) -> Result<Vec<i64>> {
    let rows = sqlx::query(
        r#"
SELECT m.version_id
FROM migrations m
JOIN (
    SELECT version_id, MAX(id) AS max_id
    FROM migrations
    GROUP BY version_id
) latest
    ON latest.version_id = m.version_id
   AND latest.max_id = m.id
WHERE m.is_applied = 1
  AND m.version_id <> 0
ORDER BY m.version_id DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<i64, _>("version_id"))
        .collect())
}

async fn latest_applied_states(pool: &SqlitePool) -> Result<HashMap<i64, bool>> {
    let rows = sqlx::query(
        r#"
SELECT m.version_id, m.is_applied
FROM migrations m
JOIN (
    SELECT version_id, MAX(id) AS max_id
    FROM migrations
    GROUP BY version_id
) latest
    ON latest.version_id = m.version_id
   AND latest.max_id = m.id
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get::<i64, _>("version_id"),
                row.get::<i64, _>("is_applied") == 1,
            )
        })
        .collect())
}
