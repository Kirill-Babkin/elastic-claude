use anyhow::{bail, Context, Result};
use std::io::{self, Read};
use crate::config::Config;

pub async fn run(
    entry_type: String,
    content: Option<String>,
    path: Option<String>,
    metadata: Option<String>,
) -> Result<()> {
    // Get content from: -c flag, -p flag (read file), or stdin
    let (content, file_path) = match (content, path) {
        (Some(c), None) => (c, None),
        (None, Some(p)) => {
            let c = std::fs::read_to_string(&p)
                .with_context(|| format!("Failed to read file: {}", p))?;
            (c, Some(p))
        }
        (None, None) => (read_stdin()?, None),
        (Some(_), Some(_)) => bail!("Cannot specify both --content and --path"),
    };

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
            &[&entry_type, &content, &file_path, &metadata_json],
        )
        .await
        .context("Failed to insert entry")?;

    let id: i32 = row.get(0);
    println!("Inserted entry with id: {}", id);

    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).context("Failed to read from stdin")?;
    Ok(buffer)
}
