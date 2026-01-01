use anyhow::{bail, Result};
use crate::docker;

pub async fn run() -> Result<()> {
    let docker_client = docker::connect().await?;
    let status = docker::get_container_status(&docker_client).await?;

    match status {
        docker::ContainerStatus::Running => {
            println!("elastic-claude is already running");
        }
        docker::ContainerStatus::Stopped => {
            println!("Starting elastic-claude...");
            docker::start_container(&docker_client).await?;
            println!("✓ elastic-claude started");
        }
        docker::ContainerStatus::NotFound => {
            bail!(
                "elastic-claude is not initialized.\n\
                 Run 'elastic-claude init' first."
            );
        }
    }

    Ok(())
}
