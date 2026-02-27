use std::path::PathBuf;
use anyhow::Result;

use crate::core::context::ContextProcessor;
use crate::core::git::CommitInfo;
use crate::utils::config::Config;

pub async fn sync_context(
    path: &PathBuf,
    config: &Config,
    from_commit: Option<String>,
    last_n: Option<usize>,
) -> Result<()> {
    let processor = ContextProcessor::new(path, config.clone())?;
    
    let commits: Vec<CommitInfo> = if let Some(from) = from_commit {
        processor.get_commit_range(&from, &processor.git.get_current_commit_hash()?)?
    } else if let Some(n) = last_n {
        processor.get_commits(n)?
    } else {
        processor.get_commits(config.context.default_commit_range)?
    };

    if commits.is_empty() {
        println!("No commits to process");
        return Ok(());
    }

    // Process oldest-first so incremental context chaining builds forward
    let mut commits = commits;
    commits.reverse();

    // Dedup: skip commits already stored
    let total_before_dedup = commits.len();
    commits.retain(|c| !processor.has_commit(&c.hash).unwrap_or(false));
    let skipped = total_before_dedup - commits.len();

    if skipped > 0 {
        println!("Skipping {} already-processed commit(s)", skipped);
    }

    if commits.is_empty() {
        println!("All commits already processed. Nothing to sync.");
        return Ok(());
    }

    println!("Processing {} new commit(s)...", commits.len());
    println!();

    if !processor.is_ollama_running() {
        return Err(anyhow::anyhow!(
            "Ollama is not running. Please start Ollama first:\n  ollama serve"
        ));
    }

    for (idx, commit) in commits.iter().enumerate() {
        println!("[{}/{}] {} - {}", idx + 1, commits.len(), &commit.short_hash,
            commit.message.lines().next().unwrap_or(""));
        log::info!("Processing commit {} ({}/{})", &commit.short_hash, idx + 1, commits.len());
        
        match processor.process_commit(commit).await {
            Ok(context) => {
                println!("  ✓ {}", context.summary);
                log::info!("  ✓ {} - {}", &commit.short_hash, context.summary);
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
                log::error!("  ✗ {} - {}", &commit.short_hash, e);
            }
        }
    }

    println!();
    let count = processor.get_context_count()?;
    println!("✓ Sync complete. Total context entries: {}", count);
    log::info!("Sync complete. Total entries: {}", count);

    Ok(())
}

pub fn get_sync_status(path: &PathBuf, config: &Config) -> Result<()> {
    let processor = ContextProcessor::new(path, config.clone())?;
    
    let total_commits = processor.git.get_commit_count()?;
    let stored_count = processor.get_context_count()?;
    let last_processed = processor.get_last_commit()?;
    
    println!("Sync Status:");
    println!("  Total commits in repo: {}", total_commits);
    println!("  Stored context entries: {}", stored_count);
    
    if let Some(last) = last_processed {
        println!("  Last processed: {}", &last[..7.min(last.len())]);
    } else {
        println!("  Last processed: None");
    }

    if processor.is_ollama_running() {
        println!("  Ollama: ✓ Running");
    } else {
        println!("  Ollama: ✗ Not running");
    }

    Ok(())
}
