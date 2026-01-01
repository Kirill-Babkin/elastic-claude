use anyhow::Result;
use std::io::{self, Write};
use crate::config::Config;
use crate::docker;

pub async fn run(include_data: bool) -> Result<()> {
    let docker_client = docker::connect().await?;
    let status = docker::get_container_status(&docker_client).await?;

    if status == docker::ContainerStatus::NotFound {
        println!("elastic-claude is not initialized");
        return Ok(());
    }

    // Confirm destruction
    print!("This will remove the elastic-claude container");
    if include_data {
        print!(" and all data");
    }
    println!(".");
    print!("Are you sure? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled");
        return Ok(());
    }

    // Stop if running
    if status == docker::ContainerStatus::Running {
        println!("Stopping container...");
        docker::stop_container(&docker_client).await?;
    }

    // Remove container
    println!("Removing container...");
    docker::remove_container(&docker_client).await?;

    // Remove volume if requested
    if include_data {
        println!("Removing data volume...");
        docker::remove_volume(&docker_client).await?;
    }

    // Remove config file
    if let Ok(config_path) = Config::config_path() {
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
            println!("Removed config file");
        }
    }

    println!("\n✓ elastic-claude destroyed");
    if !include_data {
        println!("Note: Data volume preserved. Use --include-data to remove it.");
    }

    Ok(())
}
