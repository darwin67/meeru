use std::{env, fs, path::Path};

use anyhow::{anyhow, bail, Context};
use chrono::Utc;
use meeru_storage::{migrations, StorageConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_usage();
        return Ok(());
    }

    let command = args.remove(0);
    match command.as_str() {
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        },
        "create" => create_migration(&args),
        "run" => run_migrations(&args).await,
        "rollback" => rollback_migrations(&args).await,
        "list" => list_migrations(&args).await,
        "dump" => dump_schema(&args).await,
        _ => {
            print_usage();
            bail!("unknown command: {command}");
        },
    }
}

fn create_migration(args: &[String]) -> anyhow::Result<()> {
    let name = args
        .first()
        .ok_or_else(|| anyhow!("create requires a migration name"))?;
    let slug = slugify(name);
    if slug.is_empty() {
        bail!("migration name must contain at least one alphanumeric character");
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("migrations")
        .join(format!("{timestamp}_{slug}.sql"));

    fs::write(&path, "-- +up\n\n-- +down\n")
        .with_context(|| format!("write migration file {}", path.display()))?;
    println!("{}", path.display());
    Ok(())
}

async fn run_migrations(args: &[String]) -> anyhow::Result<()> {
    let config = config_from_args(args)?;
    let storage = config
        .open_without_migrations()
        .await
        .context("open storage without applying migrations")?;
    let applied = migrations::run_migrations(storage.pool())
        .await
        .context("run migrations")?;

    if applied.is_empty() {
        println!("no migrations applied");
    } else {
        for version in applied {
            println!("applied {version}");
        }
    }
    Ok(())
}

async fn rollback_migrations(args: &[String]) -> anyhow::Result<()> {
    let (steps, remaining) = match args.first() {
        Some(value) if value.chars().all(|ch| ch.is_ascii_digit()) => (
            value
                .parse::<usize>()
                .with_context(|| format!("parse rollback count {value}"))?,
            &args[1..],
        ),
        _ => (1, args),
    };

    let config = config_from_args(remaining)?;
    let storage = config
        .open_without_migrations()
        .await
        .context("open storage without applying migrations")?;
    let rolled_back = migrations::rollback_migrations(storage.pool(), steps)
        .await
        .context("roll back migrations")?;

    if rolled_back.is_empty() {
        println!("no migrations rolled back");
    } else {
        for version in rolled_back {
            println!("rolled back {version}");
        }
    }
    Ok(())
}

async fn list_migrations(args: &[String]) -> anyhow::Result<()> {
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let cyan = "\x1b[36m";
    let reset = "\x1b[0m";

    let config = config_from_args(args)?;
    let storage = config
        .open_without_migrations()
        .await
        .context("open storage without applying migrations")?;
    let statuses = migrations::list_migrations(storage.pool())
        .await
        .context("list migrations")?;

    let version_width = statuses
        .iter()
        .map(|status| status.version.to_string().len())
        .max()
        .unwrap_or("VERSION".len())
        .max("VERSION".len());
    let status_width = "STATUS".len().max("applied".len());

    println!(
        "{cyan}{:<version_width$}  {:<status_width$}  DESCRIPTION{reset}",
        "VERSION",
        "STATUS",
        version_width = version_width,
        status_width = status_width,
    );
    println!(
        "{:-<version_width$}  {:-<status_width$}  {:-<11}",
        "",
        "",
        "",
        version_width = version_width,
        status_width = status_width,
    );

    for status in statuses {
        let state_plain = if status.applied { "applied" } else { "pending" };
        let state_colored = if status.applied {
            format!("{green}{state_plain}{reset}")
        } else {
            format!("{yellow}{state_plain}{reset}")
        };

        println!(
            "{:<version_width$}  {:<status_width$}  {}",
            status.version,
            state_colored,
            status.description,
            version_width = version_width,
            status_width = status_width,
        );
    }
    Ok(())
}

async fn dump_schema(args: &[String]) -> anyhow::Result<()> {
    let output_path = args
        .first()
        .cloned()
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

fn config_from_args(args: &[String]) -> anyhow::Result<StorageConfig> {
    match args {
        [] => StorageConfig::from_project_dirs().map_err(anyhow::Error::new),
        [root] => Ok(StorageConfig::new(root)),
        _ => bail!("expected at most one path argument"),
    }
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('_');
            last_was_separator = true;
        }
    }

    slug.trim_matches('_').to_string()
}

fn print_usage() {
    let bold = "\x1b[1m";
    let cyan = "\x1b[36m";
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let reset = "\x1b[0m";

    let command_rows = [
        (format!("{yellow}help{reset}"), "Show this help text."),
        (
            format!("{yellow}create{reset} <name>"),
            "Create a new timestamped migration file with empty +up and +down sections.",
        ),
        (
            format!("{yellow}run{reset} [storage-root]"),
            "Apply all pending migrations to the target storage database.",
        ),
        (
            format!("{yellow}rollback{reset} [count] [storage-root]"),
            "Roll back the latest applied migration, or the latest <count> migrations.",
        ),
        (
            format!("{yellow}list{reset} [storage-root]"),
            "List all known migration files and show whether each is applied or pending.",
        ),
        (
            format!("{yellow}dump{reset} [output-path]"),
            "Dump the latest executable SQLite schema shape to a SQL file.",
        ),
    ];

    let argument_rows = [
        (
            format!("{yellow}storage-root{reset}"),
            "Optional path to the Meeru storage root. Defaults to the platform app-data directory.",
        ),
        (
            format!("{yellow}output-path{reset}"),
            "Optional schema output path. Defaults to docs/generated/schema.sql.",
        ),
    ];

    let commands = format_rows(&command_rows);
    let arguments = format_rows(&argument_rows);

    eprintln!(
        "{bold}Meeru migration helper{reset}\n\
\n\
{cyan}Usage{reset}\n\
\tUse {green}scripts/migrations.sh <command>{reset} to create, run, inspect, roll back, or dump migrations.\n\
\n\
{cyan}Commands{reset}\n\
\n\
{commands}\
\n\
{cyan}Arguments{reset}\n\
\n\
{arguments}"
    );
}

fn format_rows(rows: &[(String, &str)]) -> String {
    let width = rows
        .iter()
        .map(|(label, _)| display_width(label))
        .max()
        .unwrap_or(0);

    rows.iter()
        .map(|(label, description)| {
            let padding = width.saturating_sub(display_width(label)) + 2;
            format!("\t{label}{}{}\n", " ".repeat(padding), description)
        })
        .collect()
}

fn display_width(value: &str) -> usize {
    let mut width = 0;
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            for next in chars.by_ref() {
                if next == 'm' {
                    break;
                }
            }
            continue;
        }

        width += 1;
    }

    width
}
