use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use crate::config::Config;

/// Extract plain text from JSONL chat content for proper full-text indexing.
/// Each line is a JSON object with message.content[].text structure.
pub fn extract_text_from_jsonl(raw_content: &str) -> String {
    let mut texts = Vec::new();

    for line in raw_content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<Value>(line) {
            // Extract text from message.content array
            if let Some(content_array) = json
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array())
            {
                for item in content_array {
                    // Regular text content
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        texts.push(text.to_string());
                    }
                    // Thinking content
                    if let Some(thinking) = item.get("thinking").and_then(|t| t.as_str()) {
                        texts.push(thinking.to_string());
                    }
                }
            }

            // Also extract from tool results if present
            if let Some(tool_result) = json.get("tool_result").and_then(|t| t.as_str()) {
                texts.push(tool_result.to_string());
            }
        }
    }

    texts.join("\n\n")
}

pub fn get_current_chat_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;

    // Get project key from current directory
    let current_dir = std::env::current_dir().context("Could not get current directory")?;
    let project_key = current_dir.to_string_lossy().replace("/", "-");

    // Look in the project directory for the most recent .jsonl file
    let project_dir = home.join(".claude").join("projects").join(&project_key);

    if !project_dir.exists() {
        bail!(
            "No Claude project found for: {}\nExpected at: {}",
            current_dir.display(),
            project_dir.display()
        );
    }

    // Find most recent .jsonl file in project directory
    let chat_path = find_most_recent_jsonl(&project_dir)?;

    Ok(chat_path)
}

fn find_most_recent_jsonl(dir: &PathBuf) -> Result<PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("Could not read directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "jsonl")
        })
        .collect();

    // Sort by modification time (most recent first)
    entries.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    entries
        .first()
        .map(|e| e.path())
        .context("No chat files found in project directory")
}

pub async fn run(path_only: bool, metadata: Option<String>) -> Result<()> {
    let chat_path = get_current_chat_path()?;

    if path_only {
        println!("{}", chat_path.display());
        return Ok(());
    }

    // Read the chat content
    let raw_content = std::fs::read_to_string(&chat_path)
        .with_context(|| format!("Failed to read chat file: {}", chat_path.display()))?;

    // Extract plain text from JSONL for proper full-text indexing
    let content = extract_text_from_jsonl(&raw_content);

    if content.is_empty() {
        bail!("No text content found in chat file");
    }

    let file_path = chat_path.to_string_lossy().to_string();

    // Connect to database and insert directly (to include file_path)
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

    // Parse metadata JSON if provided
    let metadata_json: serde_json::Value = match &metadata {
        Some(m) => serde_json::from_str(m).context("Invalid JSON in metadata")?,
        None => serde_json::json!({}),
    };

    let row = client
        .query_one(
            r#"
            INSERT INTO entries (entry_type, content, file_path, metadata)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            &[&"chat", &content, &Some(&file_path), &metadata_json],
        )
        .await
        .context("Failed to insert chat entry")?;

    let id: i32 = row.get(0);
    println!("Inserted chat with id: {}", id);
    println!("Chat file: {}", file_path);

    Ok(())
}
