use anyhow::{Context, Result};
use crate::config::Config;

pub async fn run(query: String) -> Result<()> {
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

    // Convert query to tsquery format (space-separated words become AND)
    let tsquery = query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" & ");

    let rows = client
        .query(
            r#"
            SELECT id, entry_type, file_path, metadata,
                   ts_headline('english', content, query,
                       'MaxFragments=3, MaxWords=30, MinWords=15, FragmentDelimiter= ... ') as snippet,
                   ts_rank(content_tsv, query) as rank
            FROM entries, to_tsquery('english', $1) query
            WHERE content_tsv @@ query
            ORDER BY rank DESC
            LIMIT 10
            "#,
            &[&tsquery],
        )
        .await
        .context("Search query failed")?;

    if rows.is_empty() {
        println!("No results found for: {}", query);
        return Ok(());
    }

    println!("Found {} results:\n", rows.len());

    for row in rows {
        let id: i32 = row.get("id");
        let entry_type: &str = row.get("entry_type");
        let file_path: Option<&str> = row.get("file_path");
        let metadata: serde_json::Value = row.get("metadata");
        let snippet: &str = row.get("snippet");
        let rank: f32 = row.get("rank");

        println!("--- Entry {} (score: {:.2}) ---", id, rank);
        println!("Type: {}", entry_type);
        if let Some(path) = file_path {
            println!("File: {}", path);
        }
        if let Some(title) = metadata.get("title").and_then(|t| t.as_str()) {
            println!("Title: {}", title);
        }
        println!("Snippet: {}...\n", snippet.trim());
    }

    Ok(())
}
