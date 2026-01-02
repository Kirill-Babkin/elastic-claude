use anyhow::{bail, Context, Result};

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

    println!("Found {} files to ingest:\n", files.len());

    for file in &files {
        println!("  {}", file.display());
    }

    println!("\nTo ingest these files, use the elastic-claude skill in Claude:");
    println!("  /elastic-claude");
    println!("\nOr manually add each file with:");
    println!("  elastic-claude add -t document -c \"$(cat <file>)\" -f \"<file>\" -m '{{\"title\": \"...\"}}'");

    Ok(())
}
