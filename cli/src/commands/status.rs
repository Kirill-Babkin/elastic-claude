use anyhow::Result;
use crate::config::Config;
use crate::docker;

pub async fn run() -> Result<()> {
    let docker_client = docker::connect().await?;
    let status = docker::get_container_status(&docker_client).await?;

    // Container status
    match status {
        docker::ContainerStatus::Running => {
            println!("Container: running ({})", docker::CONTAINER_NAME);
        }
        docker::ContainerStatus::Stopped => {
            println!("Container: stopped ({})", docker::CONTAINER_NAME);
        }
        docker::ContainerStatus::NotFound => {
            println!("Container: not found");
            println!("\nRun 'elastic-claude init' to initialize.");
            return Ok(());
        }
    }

    // Database info
    if let Ok(config) = Config::load() {
        println!(
            "Database:  {} @ {}:{}",
            config.database.name, config.database.host, config.database.port
        );

        // Get entry counts if running
        if status == docker::ContainerStatus::Running {
            if let Ok(counts) = get_entry_counts(&config).await {
                println!("Entries:   {}", counts);
            }

            if let Ok(size) = get_database_size(&config).await {
                println!("Size:      {}", size);
            }
        }

        println!("Config:    {:?}", Config::config_path()?);
    }

    Ok(())
}

async fn get_entry_counts(config: &Config) -> Result<String> {
    let (client, connection) =
        tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls).await?;

    tokio::spawn(async move {
        let _ = connection.await;
    });

    let rows = client
        .query(
            "SELECT entry_type, COUNT(*) as count FROM entries GROUP BY entry_type ORDER BY count DESC",
            &[],
        )
        .await?;

    if rows.is_empty() {
        return Ok("0 entries".to_string());
    }

    let counts: Vec<String> = rows
        .iter()
        .map(|row| {
            let entry_type: &str = row.get(0);
            let count: i64 = row.get(1);
            format!("{} {}s", count, entry_type)
        })
        .collect();

    Ok(counts.join(", "))
}

async fn get_database_size(config: &Config) -> Result<String> {
    let (client, connection) =
        tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls).await?;

    tokio::spawn(async move {
        let _ = connection.await;
    });

    let row = client
        .query_one(
            "SELECT pg_size_pretty(pg_database_size(current_database()))",
            &[],
        )
        .await?;

    Ok(row.get(0))
}
