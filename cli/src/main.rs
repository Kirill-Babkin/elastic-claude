use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod docker;

#[derive(Parser)]
#[command(name = "elastic-claude")]
#[command(about = "Local search infrastructure for project knowledge")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize elastic-claude (PostgreSQL container, config, skill)
    Init,

    /// Start the elastic-claude container
    Start,

    /// Stop the elastic-claude container
    Stop,

    /// Show status of elastic-claude
    Status,

    /// Remove elastic-claude installation
    Destroy {
        /// Also remove the data volume
        #[arg(long)]
        include_data: bool,
    },

    /// Ingest files into the knowledge base
    Ingest {
        /// Glob patterns for files to ingest
        #[arg(required = true)]
        patterns: Vec<String>,
    },

    /// Search the knowledge base
    Search {
        /// Search query
        #[arg(required = true)]
        query: String,
    },

    /// Ingest a chat session
    Chat {
        /// Path to chat session file
        #[arg(required = true)]
        session_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run().await,
        Commands::Start => commands::start::run().await,
        Commands::Stop => commands::stop::run().await,
        Commands::Status => commands::status::run().await,
        Commands::Destroy { include_data } => commands::destroy::run(include_data).await,
        Commands::Ingest { patterns } => commands::ingest::run(patterns).await,
        Commands::Search { query } => commands::search::run(query).await,
        Commands::Chat { session_file } => commands::chat::run(session_file).await,
    }
}
