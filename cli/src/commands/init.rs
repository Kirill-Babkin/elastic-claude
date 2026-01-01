use anyhow::{Context, Result};
use crate::config::Config;
use crate::docker;

const INIT_SQL: &str = include_str!("../../../docker/init.sql");

pub async fn run() -> Result<()> {
    println!("Initializing elastic-claude...\n");

    // Connect to Docker
    let docker_client = docker::connect().await?;

    // Check if already initialized
    let status = docker::get_container_status(&docker_client).await?;
    if status != docker::ContainerStatus::NotFound {
        println!("elastic-claude is already initialized.");
        println!("Use 'elastic-claude start' to start the container.");
        return Ok(());
    }

    // Pull image
    docker::pull_image(&docker_client).await?;

    // Create volume
    docker::create_volume(&docker_client).await?;

    // Create config
    let config = Config::default();

    // Create and start container
    docker::create_container(
        &docker_client,
        &config.database.password,
        &config.database.name,
    )
    .await?;

    println!("Starting container...");
    docker::start_container(&docker_client).await?;

    // Wait for PostgreSQL
    docker::wait_for_postgres(&config).await?;

    // Run migrations
    println!("Running database migrations...");
    run_migrations(&config).await?;

    // Save config
    config.save()?;
    println!("Config saved to {:?}", Config::config_path()?);

    // Install skill
    install_skill()?;

    println!("\n✓ elastic-claude initialized successfully!");
    println!("\nDatabase: {} @ localhost:{}", config.database.name, config.database.port);
    println!("Config:   {:?}", Config::config_path()?);
    println!("Skill:    {:?}", Config::skill_dir()?);

    Ok(())
}

async fn run_migrations(config: &Config) -> Result<()> {
    let (client, connection) =
        tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls)
            .await
            .context("Failed to connect to database")?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    client
        .batch_execute(INIT_SQL)
        .await
        .context("Failed to run migrations")?;

    println!("Migrations completed successfully");
    Ok(())
}

fn install_skill() -> Result<()> {
    let skill_dir = Config::skill_dir()?;
    std::fs::create_dir_all(&skill_dir)?;

    // Copy SKILL.md
    let skill_content = include_str!("../../../skill/SKILL.md");
    std::fs::write(skill_dir.join("SKILL.md"), skill_content)?;

    // Copy references
    let refs_dir = skill_dir.join("references");
    std::fs::create_dir_all(&refs_dir)?;

    let schema_content = include_str!("../../../skill/references/schema.md");
    std::fs::write(refs_dir.join("schema.md"), schema_content)?;

    println!("Skill installed to {:?}", skill_dir);
    Ok(())
}
