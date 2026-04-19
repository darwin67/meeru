use std::{env, fs, path::Path};

use anyhow::Context;
use meeru_storage::StorageConfig;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let output_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/generated/schema.sql".to_string());

    let temp_root = env::temp_dir().join(format!("meeru-schema-dump-{}", Uuid::new_v4()));

    let schema = match StorageConfig::new(&temp_root).open().await {
        Ok(storage) => dump_schema(storage.pool())
            .await
            .context("dump schema from migrated sqlite database"),
        Err(error) => Err(anyhow::Error::new(error).context("open temporary storage root")),
    };

    let _ = fs::remove_dir_all(&temp_root);

    let schema = schema?;
    if let Some(parent) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent directories for {}", output_path))?;
    }

    fs::write(&output_path, schema)
        .with_context(|| format!("write schema dump to {}", output_path))?;

    println!("{output_path}");
    Ok(())
}

async fn dump_schema(pool: &SqlitePool) -> anyhow::Result<String> {
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
    .await
    .context("query sqlite_master")?;

    let mut output = String::from(
        "-- Generated from the executable meeru-storage SQLite migrations.\n\
-- Do not edit by hand; regenerate with `make dump-schema`.\n\n",
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
