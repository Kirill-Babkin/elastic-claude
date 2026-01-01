use anyhow::{Context, Result};
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, StartContainerOptions, StopContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::volume::CreateVolumeOptions;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;

pub const CONTAINER_NAME: &str = "elastic-claude-db";
pub const VOLUME_NAME: &str = "elastic-claude-data";
pub const IMAGE_NAME: &str = "postgres:16-alpine";

pub async fn connect() -> Result<Docker> {
    Docker::connect_with_local_defaults().context(
        "Could not connect to Docker. Is Docker installed and running?\n\
         Install: https://docs.docker.com/get-docker/",
    )
}

pub async fn pull_image(docker: &Docker) -> Result<()> {
    println!("Pulling {}...", IMAGE_NAME);

    let options = CreateImageOptions {
        from_image: IMAGE_NAME,
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(options), None, None);
    while let Some(result) = stream.next().await {
        result.context("Failed to pull image")?;
    }

    println!("Image pulled successfully");
    Ok(())
}

pub async fn create_volume(docker: &Docker) -> Result<()> {
    println!("Creating volume {}...", VOLUME_NAME);

    let options = CreateVolumeOptions {
        name: VOLUME_NAME,
        ..Default::default()
    };

    docker.create_volume(options).await.context("Failed to create volume")?;
    println!("Volume created successfully");
    Ok(())
}

pub async fn create_container(docker: &Docker, password: &str, db_name: &str) -> Result<()> {
    println!("Creating container {}...", CONTAINER_NAME);

    let options = CreateContainerOptions {
        name: CONTAINER_NAME,
        ..Default::default()
    };

    let host_config = bollard::service::HostConfig {
        port_bindings: Some({
            let mut map = HashMap::new();
            map.insert(
                "5432/tcp".to_string(),
                Some(vec![bollard::service::PortBinding {
                    host_ip: Some("127.0.0.1".to_string()),
                    host_port: Some("5433".to_string()),
                }]),
            );
            map
        }),
        binds: Some(vec![format!("{}:/var/lib/postgresql/data", VOLUME_NAME)]),
        ..Default::default()
    };

    let config = Config {
        image: Some(IMAGE_NAME),
        env: Some(vec![
            &format!("POSTGRES_PASSWORD={}", password),
            &format!("POSTGRES_DB={}", db_name),
        ]),
        host_config: Some(host_config),
        ..Default::default()
    };

    docker
        .create_container(Some(options), config)
        .await
        .context("Failed to create container")?;

    println!("Container created successfully");
    Ok(())
}

pub async fn start_container(docker: &Docker) -> Result<()> {
    docker
        .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
        .await
        .context("Failed to start container")?;
    Ok(())
}

pub async fn stop_container(docker: &Docker) -> Result<()> {
    docker
        .stop_container(CONTAINER_NAME, Some(StopContainerOptions { t: 10 }))
        .await
        .context("Failed to stop container")?;
    Ok(())
}

pub async fn remove_container(docker: &Docker) -> Result<()> {
    docker
        .remove_container(CONTAINER_NAME, None)
        .await
        .context("Failed to remove container")?;
    Ok(())
}

pub async fn remove_volume(docker: &Docker) -> Result<()> {
    docker
        .remove_volume(VOLUME_NAME, None)
        .await
        .context("Failed to remove volume")?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStatus {
    Running,
    Stopped,
    NotFound,
}

pub async fn get_container_status(docker: &Docker) -> Result<ContainerStatus> {
    let options = ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;

    for container in containers {
        if let Some(names) = container.names {
            if names.iter().any(|n| n == &format!("/{}", CONTAINER_NAME)) {
                return match container.state.as_deref() {
                    Some("running") => Ok(ContainerStatus::Running),
                    _ => Ok(ContainerStatus::Stopped),
                };
            }
        }
    }

    Ok(ContainerStatus::NotFound)
}

pub async fn wait_for_postgres(config: &crate::config::Config) -> Result<()> {
    println!("Waiting for PostgreSQL to be ready...");

    for i in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        match tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls).await {
            Ok(_) => {
                println!("PostgreSQL is ready");
                return Ok(());
            }
            Err(_) if i < 29 => continue,
            Err(e) => return Err(e).context("PostgreSQL did not become ready in time"),
        }
    }

    Ok(())
}
