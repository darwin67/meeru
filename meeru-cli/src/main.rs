//! Meeru CLI Application

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "meeru")]
#[command(about = "A unified email client CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage email accounts
    Accounts {
        #[command(subcommand)]
        command: AccountCommands,
    },
    /// Email operations
    Email {
        #[command(subcommand)]
        command: EmailCommands,
    },
    /// Start the API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum AccountCommands {
    /// List all accounts
    List,
    /// Add a new account
    Add {
        /// Email address
        email: String,
    },
    /// Remove an account
    Remove {
        /// Account ID or email
        account: String,
    },
}

#[derive(Subcommand)]
enum EmailCommands {
    /// List emails
    List {
        /// Filter by unread
        #[arg(long)]
        unread: bool,
        /// Limit number of results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Send an email
    Send {
        /// Recipient email
        to: String,
        /// Email subject
        #[arg(short, long)]
        subject: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    meeru_core::logging::init_logging();

    let cli = Cli::parse();

    match cli.command {
        Commands::Accounts { command } => match command {
            AccountCommands::List => {
                tracing::info!("Listing accounts");
                println!("Listing accounts...");
            },
            AccountCommands::Add { email } => {
                tracing::info!(email = %email, "Adding account");
                println!("Adding account: {}", email);
            },
            AccountCommands::Remove { account } => {
                tracing::info!(account = %account, "Removing account");
                println!("Removing account: {}", account);
            },
        },
        Commands::Email { command } => match command {
            EmailCommands::List { unread, limit } => {
                println!("Listing emails (unread: {}, limit: {})", unread, limit);
            },
            EmailCommands::Send { to, subject } => {
                println!("Sending email to {} with subject: {}", to, subject);
            },
        },
        Commands::Serve { port } => {
            println!("Starting API server on port {}", port);
            // API server implementation will go here
        },
    }

    Ok(())
}
