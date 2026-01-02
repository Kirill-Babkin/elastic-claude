use anyhow::Result;
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

    /// Add an entry to the knowledge base (used by skill)
    Add {
        /// Entry type (e.g., "document", "chat", "code")
        #[arg(short = 't', long)]
        entry_type: String,

        /// Content to store (inline)
        #[arg(short, long, conflicts_with = "path")]
        content: Option<String>,

        /// Read content from file path
        #[arg(short, long, conflicts_with = "content")]
        path: Option<String>,

        /// JSON metadata (optional)
        #[arg(short, long)]
        metadata: Option<String>,
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

    /// Find and optionally ingest current Claude chat session
    CurrentChat {
        /// Only print the path, don't ingest
        #[arg(long)]
        path_only: bool,

        /// JSON metadata (optional, for ingestion)
        #[arg(short, long)]
        metadata: Option<String>,
    },

    /// Get an entry by ID
    Get {
        /// Entry ID
        id: i32,

        /// Only print the content
        #[arg(long)]
        content_only: bool,

        /// Show the tsvector (full-text search tokens)
        #[arg(long)]
        tsv: bool,
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
        Commands::Add { entry_type, content, path, metadata } => {
            commands::add::run(entry_type, content, path, metadata).await
        }
        Commands::Ingest { patterns } => commands::ingest::run(patterns).await,
        Commands::Search { query } => commands::search::run(query).await,
        Commands::Chat { session_file } => commands::chat::run(session_file).await,
        Commands::CurrentChat { path_only, metadata } => {
            commands::current_chat::run(path_only, metadata).await
        }
        Commands::Get { id, content_only, tsv } => {
            commands::get::run(id, content_only, tsv).await
        }
    }
}
