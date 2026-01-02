use anyhow::{Context, Result};
use crate::config::Config;

pub async fn run(id: i32, content_only: bool, show_tsv: bool) -> Result<()> {
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

    let query = if show_tsv {
        r#"
        SELECT id, entry_type, content, content_tsv::text, file_path, metadata, created_at
        FROM entries
        WHERE id = $1
        "#
    } else {
        r#"
        SELECT id, entry_type, content, NULL::text as content_tsv, file_path, metadata, created_at
        FROM entries
        WHERE id = $1
        "#
    };

    let row = client
        .query_opt(query, &[&id])
        .await
        .context("Failed to query entry")?;

    match row {
        None => {
            println!("Entry {} not found", id);
        }
        Some(row) => {
            let content: &str = row.get("content");

            if content_only {
                println!("{}", content);
                return Ok(());
            }

            let entry_type: &str = row.get("entry_type");
            let file_path: Option<&str> = row.get("file_path");
            let metadata: serde_json::Value = row.get("metadata");
            let created_at: chrono::NaiveDateTime = row.get("created_at");

            println!("=== Entry {} ===", id);
            println!("Type: {}", entry_type);
            if let Some(path) = file_path {
                println!("File: {}", path);
            }
            println!("Created: {}", created_at);
            println!("Metadata: {}", serde_json::to_string_pretty(&metadata)?);

            if show_tsv {
                let tsv: Option<&str> = row.get("content_tsv");
                if let Some(tsv_text) = tsv {
                    println!("\n--- TSVector ---");
                    println!("{}", tsv_text);
                }
            }

            println!("\n--- Content ---");
            println!("{}", content);
        }
    }

    Ok(())
}
