use anyhow::{bail, Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn run(query: String) -> Result<()> {
    let prompt = format!(
        r#"Use the elastic-claude skill to search for: {}

Return relevant entries with snippets and metadata."#,
        query
    );

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
