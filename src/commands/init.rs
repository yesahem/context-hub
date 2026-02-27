use anyhow::Result;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::core::git::GitAnalyzer;
use crate::core::llm;
use crate::core::storage::Storage;
use crate::utils::config::Config;

pub fn init_repo(path: &PathBuf) -> Result<()> {
    println!("\n🚀 Initializing ContextHub...\n");

    // 1. Check git FIRST before creating any directories
    let _git = GitAnalyzer::new(path).map_err(|_| {
        anyhow::anyhow!("Not a git repository. Run 'git init' first.")
    })?;
    println!("  ✓ Git repository detected");

    let context_dir = path.join(".contexthub");
    if context_dir.exists() {
        println!("  ⚠️  ContextHub already initialized in this directory");
        println!("  Run 'contexthub config show' to view current config.");
        return Ok(());
    }

    // 2. Create directory structure
    std::fs::create_dir_all(&context_dir)?;
    std::fs::create_dir_all(context_dir.join("memory/ttl"))?;
    std::fs::create_dir_all(context_dir.join("memory/global"))?;
    std::fs::create_dir_all(context_dir.join("cache"))?;
    std::fs::create_dir_all(context_dir.join("logs"))?;
    println!("  ✓ Created .contexthub/ directory");

    // 3. Initialize SQLite database
    let _storage = Storage::new(&context_dir.join("context.db"))?;
    println!("  ✓ Initialized SQLite database");

    // 4. Add .contexthub/ to .gitignore
    add_to_gitignore(path)?;
    println!("  ✓ Added .contexthub/ to .gitignore");

    // 5. Interactive Ollama configuration
    println!();
    let config = configure_ollama_interactive()?;
    config.save(path)?;
    println!("  ✓ Configuration saved");

    println!("\n🎉 ContextHub initialized successfully!\n");
    println!("Next steps:");
    println!("  contexthub sync          - Extract context from commits");
    println!("  contexthub context       - View stored context");
    println!("  contexthub hook install  - Enable auto-sync on commit");
    println!();

    Ok(())
}

/// Interactive Ollama setup during init
fn configure_ollama_interactive() -> Result<Config> {
    let mut config = Config::default();

    println!("── Ollama Configuration ──────────────────────────");
    println!();

    // Ask for endpoint
    let endpoint = prompt_with_default(
        "Ollama endpoint",
        &config.ollama.endpoint,
    )?;
    config.ollama.endpoint = endpoint;

    // Check if Ollama is reachable
    print!("  Checking Ollama... ");
    io::stdout().flush()?;

    let ollama_running = reqwest::blocking::get(format!("{}/api/tags", config.ollama.endpoint))
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !ollama_running {
        println!("✗ Not running");
        println!();
        println!("  Ollama is not reachable at {}", config.ollama.endpoint);
        println!("  Start it later with: ollama serve");
        println!("  Using default model: {}", config.ollama.model);
        println!("  You can change it anytime: contexthub config set-model <model>");
        println!();
        return Ok(config);
    }
    println!("✓ Running");

    // Fetch available models
    match llm::fetch_available_models(&config.ollama.endpoint) {
        Ok(models) if !models.is_empty() => {
            println!();
            println!("  Available models:");
            println!();
            for (i, model) in models.iter().enumerate() {
                let marker = if *model == config.ollama.model || model.starts_with(&config.ollama.model) {
                    " (default)"
                } else {
                    ""
                };
                println!("    {}. {}{}", i + 1, model, marker);
            }
            println!();

            let choice = prompt_with_default(
                &format!("Select model [1-{}] or enter name", models.len()),
                "1",
            )?;

            let selected = if let Ok(idx) = choice.parse::<usize>() {
                if idx >= 1 && idx <= models.len() {
                    models[idx - 1].clone()
                } else {
                    println!("  Invalid selection, using first model.");
                    models[0].clone()
                }
            } else if !choice.is_empty() {
                choice // User typed a custom model name
            } else {
                models[0].clone()
            };

            config.ollama.model = selected;
            println!("  ✓ Model set to: {}", config.ollama.model);
        }
        Ok(_) => {
            println!("  No models found. Pull one first:");
            println!("    ollama pull llama3.2");
            println!("  Using default: {}", config.ollama.model);
        }
        Err(e) => {
            println!("  Could not fetch models: {}", e);
            println!("  Using default: {}", config.ollama.model);
        }
    }

    println!();
    Ok(config)
}

/// Prompt the user with a default value. Returns the default if they just press Enter.
fn prompt_with_default(label: &str, default: &str) -> Result<String> {
    print!("  {} [{}]: ", label, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
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
