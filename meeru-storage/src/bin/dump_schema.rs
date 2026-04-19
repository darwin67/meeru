use std::{env, fs, path::Path};

use anyhow::Context;
use meeru_storage::{migrations, StorageConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let output_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/generated/schema.sql".to_string());

    let temp_root = env::temp_dir().join(format!("meeru-schema-dump-{}", Uuid::new_v4()));

    let schema = match StorageConfig::new(&temp_root).open().await {
        Ok(storage) => migrations::dump_schema(storage.pool())
            .await
            .map_err(anyhow::Error::new)
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
