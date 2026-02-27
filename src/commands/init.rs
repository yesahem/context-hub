use anyhow::Result;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::core::git::GitAnalyzer;
use crate::core::llm;
use crate::core::storage::Storage;
use crate::utils::config::Config;

pub async fn init_repo(path: &PathBuf) -> Result<()> {
    println!();
    println!("  \x1b[1;36m╔═══════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m   🚀 \x1b[1mContextHub Setup Wizard\x1b[0m          \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╚═══════════════════════════════════════╝\x1b[0m");
    println!();

    // ── Step 1: Validate git repo ────────────────────────────
    print!("  Checking git repository... ");
    io::stdout().flush()?;
    let _git = GitAnalyzer::new(path).map_err(|_| {
        anyhow::anyhow!("Not a git repository. Run 'git init' first.")
    })?;
    println!("✓");

    let context_dir = path.join(".contexthub");
    if context_dir.exists() {
        println!();
        println!("  ⚠️  ContextHub already initialized in this directory.");
        println!("  Run 'contexthub config show' to view current config.");
        return Ok(());
    }

    // ── Step 2: Create directory + DB ────────────────────────
    print!("  Creating .contexthub/ directory... ");
    io::stdout().flush()?;
    std::fs::create_dir_all(&context_dir)?;
    std::fs::create_dir_all(context_dir.join("memory/ttl"))?;
    std::fs::create_dir_all(context_dir.join("memory/global"))?;
    std::fs::create_dir_all(context_dir.join("cache"))?;
    std::fs::create_dir_all(context_dir.join("logs"))?;
    println!("✓");

    print!("  Initializing SQLite database... ");
    io::stdout().flush()?;
    let _storage = Storage::new(&context_dir.join("context.db"))?;
    println!("✓");

    print!("  Adding .contexthub/ to .gitignore... ");
    io::stdout().flush()?;
    add_to_gitignore(path)?;
    println!("✓");

    // ── Step 3: Ollama model selection ───────────────────────
    println!();
    println!("  \x1b[1m── Step 1/3: Ollama Configuration ──\x1b[0m");
    println!();

    let mut config = Config::default();

    let endpoint = prompt_with_default(
        "Ollama endpoint",
        &config.ollama.endpoint,
    )?;
    config.ollama.endpoint = endpoint;

    print!("  Checking Ollama... ");
    io::stdout().flush()?;

    let ollama_running = reqwest::blocking::get(format!("{}/api/tags", config.ollama.endpoint))
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !ollama_running {
        println!("✗ Not running");
        println!();
        println!("  ⚠️  Ollama is not reachable at {}", config.ollama.endpoint);
        println!("  You'll need to start it before syncing: ollama serve");
        println!("  Using default model: {}", config.ollama.model);
        config.save(path)?;
        print_final_summary(path, &config, false, false);
        return Ok(());
    }
    println!("✓ Running");

    match llm::fetch_available_models(&config.ollama.endpoint) {
        Ok(models) if !models.is_empty() => {
            println!();
            println!("  Available models:");
            println!();
            for (i, model) in models.iter().enumerate() {
                println!("    \x1b[1;33m{}.\x1b[0m {}", i + 1, model);
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
                choice
            } else {
                models[0].clone()
            };

            config.ollama.model = selected;
            println!("  ✓ Model: \x1b[1;32m{}\x1b[0m", config.ollama.model);
        }
        Ok(_) => {
            println!("  No models found locally. Pull one with: ollama pull llama3.2");
            println!("  Using default: {}", config.ollama.model);
        }
        Err(e) => {
            println!("  Could not fetch models: {}", e);
            println!("  Using default: {}", config.ollama.model);
        }
    }

    // ── Step 4: Auto-sync hook ───────────────────────────────
    println!();
    println!("  \x1b[1m── Step 2/3: Git Hook (Auto-Sync) ──\x1b[0m");
    println!();
    println!("  The post-commit hook will automatically extract context");
    println!("  after every git commit. This runs in the background but");
    println!("  \x1b[1;33mmay add a brief delay\x1b[0m after each commit while Ollama processes.");
    println!();

    let install_hook = prompt_yes_no("  Enable auto-sync hook?", false)?;

    let mut hook_installed = false;
    if install_hook {
        match crate::commands::hook::install_hook(path) {
            Ok(()) => {
                config.git.hook_enabled = true;
                config.git.auto_sync = true;
                hook_installed = true;
            }
            Err(e) => {
                println!("  ⚠️  Could not install hook: {}", e);
                println!("  You can install it later: contexthub hook install");
            }
        }
    } else {
        println!("  Skipped. You can enable it later: contexthub hook install");
    }

    // Save config before sync (in case sync fails)
    config.save(path)?;

    // ── Step 5: Initial sync ─────────────────────────────────
    println!();
    println!("  \x1b[1m── Step 3/3: Initial Sync ──\x1b[0m");
    println!();

    let git = GitAnalyzer::new(path)?;
    let commit_count = git.get_commit_count().unwrap_or(0);
    let sync_count = commit_count.min(config.context.default_commit_range);

    if commit_count == 0 {
        println!("  No commits in this repo yet. Sync will run after your first commit.");
    } else {
        println!("  This repo has \x1b[1m{}\x1b[0m commit(s).", commit_count);
        println!("  ContextHub will process the last \x1b[1m{}\x1b[0m commit(s).", sync_count);
        println!();
        println!("  \x1b[1;33m⚠ Note:\x1b[0m Each commit is sent to Ollama for analysis.");
        println!("  This may take \x1b[1m~10-30 seconds per commit\x1b[0m depending on your hardware.");
        if sync_count > 5 {
            let est_min = sync_count * 10 / 60;
            let est_max = sync_count * 30 / 60;
            println!("  Estimated time: \x1b[1m{}-{} minutes\x1b[0m for {} commits.",
                est_min.max(1), est_max.max(1), sync_count);
        }
        println!();

        let do_sync = prompt_yes_no("  Run initial sync now?", true)?;

        if do_sync {
            println!();
            // Run sync inline
            match crate::commands::sync::sync_context(
                path,
                &config,
                None,
                Some(sync_count),
            ).await {
                Ok(()) => {}
                Err(e) => {
                    println!();
                    println!("  ⚠️  Sync encountered an error: {}", e);
                    println!("  You can retry later: contexthub sync");
                }
            }
        } else {
            println!("  Skipped. Run it anytime: contexthub sync");
        }
    }

    print_final_summary(path, &config, hook_installed, commit_count > 0);
    Ok(())
}

fn print_final_summary(_path: &PathBuf, config: &Config, hook_installed: bool, has_commits: bool) {
    println!();
    println!("  \x1b[1;36m╔═══════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║\x1b[0m   🎉 \x1b[1mContextHub Ready!\x1b[0m               \x1b[1;36m║\x1b[0m");
    println!("  \x1b[1;36m╚═══════════════════════════════════════╝\x1b[0m");
    println!();
    println!("  \x1b[1mConfiguration:\x1b[0m");
    println!("    Model:     {}", config.ollama.model);
    println!("    Endpoint:  {}", config.ollama.endpoint);
    println!("    Auto-sync: {}", if hook_installed { "✓ enabled" } else { "✗ disabled" });
    println!("    TTL:       {} days", config.context.ttl_days);
    println!();
    println!("  \x1b[1mUseful commands:\x1b[0m");
    if has_commits {
        println!("    contexthub sync            Sync more commits");
        println!("    contexthub context         View stored context");
        println!("    contexthub context -e claude   Export as CLAUDE.md");
        println!("    contexthub memory          View TTL memory");
    } else {
        println!("    contexthub sync            Sync commits (after your first commit)");
        println!("    contexthub context         View stored context");
    }
    if !hook_installed {
        println!("    contexthub hook install     Enable auto-sync");
    }
    println!("    contexthub doctor           Check system health");
    println!("    contexthub status           View sync status");
    println!();
}

/// Ask a yes/no question. Returns true for yes.
fn prompt_yes_no(label: &str, default_yes: bool) -> Result<bool> {
    let hint = if default_yes { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", label, hint);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        Ok(default_yes)
    } else {
        Ok(input == "y" || input == "yes")
    }
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
