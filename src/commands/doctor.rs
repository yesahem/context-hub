use anyhow::Result;
use std::path::PathBuf;

use crate::core::llm::check_ollama_installation;
use crate::utils::config::Config;

pub fn doctor(path: &PathBuf, config: &Config) -> Result<()> {
    println!("🔍 System Health Check\n");

    // Git check
    print!("  Git: ");
    match crate::core::git::GitAnalyzer::new(path) {
        Ok(git) => {
            let commit_count = git.get_commit_count()?;
            println!("✓ Repository found ({} commits)", commit_count);
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    // Ollama installation
    print!("  Ollama (installation): ");
    if check_ollama_installation() {
        println!("✓ Installed");
    } else {
        println!("✗ Not found - install from https://ollama.ai");
    }

    // Ollama running
    print!("  Ollama (running): ");
    let llm = crate::core::llm::LlmProcessor::new(config.ollama.clone());
    if llm.is_ollama_running() {
        println!("✓ Running at {}", config.ollama.endpoint);
    } else {
        println!("✗ Not running - start with 'ollama serve'");
    }

    // ContextHub initialized
    print!("  ContextHub initialized: ");
    if path.join(".contexthub").exists() {
        let count = std::fs::read_dir(path.join(".contexthub"))?.count();
        println!("✓ Yes ({} items)", count);
    } else {
        println!("✗ No - run 'contexthub init'");
    }

    // Database
    print!("  Database: ");
    let db_path = path.join(".contexthub/context.db");
    if db_path.exists() {
        println!("✓ Exists");
    } else {
        println!("✗ Not found");
    }

    println!();
    println!("📝 Recommendations:");
    let mut rec = 1;

    if !check_ollama_installation() {
        println!("  {}. Install Ollama: curl -fsSL https://ollama.ai/install.sh | sh", rec);
        rec += 1;
    }

    if !llm.is_ollama_running() {
        println!("  {}. Start Ollama: ollama serve", rec);
        rec += 1;
    }

    if !path.join(".contexthub").exists() {
        println!("  {}. Initialize: contexthub init", rec);
        rec += 1;
    }

    if rec == 1 {
        println!("  All good! No issues found.");
    }

    Ok(())
}
