use anyhow::Result;
use crate::docker;

pub async fn run() -> Result<()> {
    let docker_client = docker::connect().await?;
    let status = docker::get_container_status(&docker_client).await?;

    match status {
        docker::ContainerStatus::Running => {
            println!("Stopping elastic-claude...");
            docker::stop_container(&docker_client).await?;
            println!("✓ elastic-claude stopped");
        }
        docker::ContainerStatus::Stopped => {
            println!("elastic-claude is already stopped");
        }
        docker::ContainerStatus::NotFound => {
            println!("elastic-claude is not initialized");
        }
    }

    Ok(())
}
