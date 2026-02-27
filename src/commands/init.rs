use anyhow::Result;
use std::path::PathBuf;

use crate::core::git::GitAnalyzer;
use crate::core::storage::Storage;
use crate::utils::config::Config;

pub fn init_repo(path: &PathBuf) -> Result<()> {
    println!("Initializing ContextHub in {}...", path.display());

    // 1. Check git FIRST before creating any directories
    let _git = GitAnalyzer::new(path).map_err(|_| {
        anyhow::anyhow!("Not a git repository. Run 'git init' first.")
    })?;

    let context_dir = path.join(".contexthub");
    if context_dir.exists() {
        println!("⚠️  ContextHub already initialized in this directory");
        return Ok(());
    }

    // 2. Create directory structure
    std::fs::create_dir_all(&context_dir)?;
    std::fs::create_dir_all(context_dir.join("memory/ttl"))?;
    std::fs::create_dir_all(context_dir.join("memory/global"))?;
    std::fs::create_dir_all(context_dir.join("cache"))?;
    std::fs::create_dir_all(context_dir.join("logs"))?;
    println!("  ✓ Created .contexthub/ directory");

    // 3. Save default config
    let config = Config::default();
    config.save(path)?;
    println!("  ✓ Configuration saved");

    // 4. Actually create the SQLite database
    let _storage = Storage::new(&context_dir.join("context.db"))?;
    println!("  ✓ Initialized SQLite database");

    // 5. Add .contexthub/ to .gitignore
    add_to_gitignore(path)?;
    println!("  ✓ Added .contexthub/ to .gitignore");

    println!();
    println!("🎉 ContextHub initialized successfully!");
    println!();
    println!("Next steps:");
    println!("  contexthub sync          - Extract context from commits");
    println!("  contexthub context       - View stored context");
    println!("  contexthub hook install  - Enable auto-sync on commit");

    Ok(())
}

pub fn is_initialized(path: &PathBuf) -> bool {
    path.join(".contexthub/config.json").exists()
}

/// Ensures `.contexthub/` is in `.gitignore`. Creates the file if missing.
fn add_to_gitignore(repo_path: &PathBuf) -> Result<()> {
    let gitignore_path = repo_path.join(".gitignore");
    let entry = ".contexthub/";

    if gitignore_path.exists() {
        let content = std::fs::read_to_string(&gitignore_path)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(()); // Already present
        }
        // Append with a newline separator
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        std::fs::write(
            &gitignore_path,
            format!("{}{}\n# ContextHub local data\n{}\n", content, separator, entry),
        )?;
    } else {
        std::fs::write(
            &gitignore_path,
            format!("# ContextHub local data\n{}\n", entry),
        )?;
    }

    Ok(())
}
