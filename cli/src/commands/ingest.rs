use anyhow::{bail, Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn run(patterns: Vec<String>) -> Result<()> {
    // Resolve glob patterns to file list
    let mut files = Vec::new();
    for pattern in &patterns {
        for entry in glob::glob(pattern).context("Invalid glob pattern")? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        files.push(path);
                    }
                }
                Err(e) => eprintln!("Warning: {}", e),
            }
        }
    }

    if files.is_empty() {
        bail!("No files found matching the patterns");
    }

    println!("Found {} files to ingest", files.len());

    // Build file list for prompt
    let file_list: Vec<String> = files
        .iter()
        .map(|p| p.display().to_string())
        .collect();

    let prompt = format!(
        r#"Use the elastic-claude skill to ingest documents.
Files: {}

For each file:
  - Read content
  - Extract metadata based on content and path
  - Insert into database"#,
        file_list.join("\n")
    );

    // Spawn Claude Code
    spawn_claude(&prompt).await
}

async fn spawn_claude(prompt: &str) -> Result<()> {
    let mut child = Command::new("claude")
        .arg("--print")
        .arg(prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn Claude Code. Is it installed?")?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line? {
                    Some(line) => println!("{}", line),
                    None => break,
                }
            }
            line = stderr_lines.next_line() => {
                match line? {
                    Some(line) => eprintln!("{}", line),
                    None => {}
                }
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        bail!("Claude Code exited with error");
    }

    Ok(())
}
