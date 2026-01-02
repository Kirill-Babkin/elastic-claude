use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use crate::config::Config;
use super::current_chat::extract_text_from_jsonl;

pub async fn run(session_file: PathBuf) -> Result<()> {
    if !session_file.exists() {
        bail!("Session file not found: {}", session_file.display());
    }

    // Read the chat content
    let raw_content = std::fs::read_to_string(&session_file)
        .with_context(|| format!("Failed to read chat file: {}", session_file.display()))?;

    // Extract plain text from JSONL for proper full-text indexing
    let content = extract_text_from_jsonl(&raw_content);

    if content.is_empty() {
        bail!("No text content found in chat file");
    }

    let file_path = session_file.to_string_lossy().to_string();

    // Connect to database and insert
    let config = Config::load().context("elastic-claude not initialized. Run 'elastic-claude init' first.")?;

    let (client, connection) =
        tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls)
            .await
            .context("Failed to connect to database")?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    let row = client
        .query_one(
            r#"
            INSERT INTO entries (entry_type, content, file_path, metadata)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            &[&"chat", &content, &Some(&file_path), &serde_json::json!({})],
        )
        .await
        .context("Failed to insert chat entry")?;

    let id: i32 = row.get(0);
    println!("Inserted chat with id: {}", id);
    println!("Chat file: {}", file_path);

    Ok(())
}
