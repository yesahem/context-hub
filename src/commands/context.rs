use anyhow::Result;
use std::path::PathBuf;

use crate::core::context::ContextProcessor;
use crate::utils::config::Config;

pub fn display_context(path: &PathBuf, config: &Config) -> Result<()> {
    let processor = ContextProcessor::new(path, config.clone())?;
    let contexts = processor.get_global_context()?;

    if contexts.is_empty() {
        println!("No context stored. Run 'contexthub sync' first.");
        return Ok(());
    }

    println!("📚 Global Context ({} entries)\n", contexts.len());

    for ctx in contexts.iter().take(20) {
        println!("┌─ {} ─", &ctx.commit_hash[..7.min(ctx.commit_hash.len())]);
        println!(
            "│ {}",
            ctx.commit_message.lines().next().unwrap_or("No message")
        );
        println!("│ {}", ctx.context_summary);
        if !ctx.files_changed.is_empty() {
            let files: Vec<String> = serde_json::from_str(&ctx.files_changed).unwrap_or_default();
            println!("│ Files: {}", files.join(", "));
        }
        println!("└─ {} ─", ctx.commit_date.format("%Y-%m-%d %H:%M"));
        println!();
    }

    Ok(())
}

pub fn export_context(path: &PathBuf, config: &Config, format: &str) -> Result<()> {
    let processor = ContextProcessor::new(path, config.clone())?;

    let output = match format {
        "markdown" | "md" => processor.export_context_markdown()?,
        "json" => processor.export_context_json()?,
        "claude" => {
            let content = processor.export_for_claude()?;
            let out_path = path.join("CLAUDE.md");
            std::fs::write(&out_path, &content)?;
            println!("✓ Exported to {}", out_path.display());
            return Ok(());
        }
        "cursor" | "cursorrules" => {
            let content = processor.export_for_cursor()?;
            let out_path = path.join(".cursorrules");
            std::fs::write(&out_path, &content)?;
            println!("✓ Exported to {}", out_path.display());
            return Ok(());
        }
        "copilot" | "github-copilot" => {
            let content = processor.export_for_copilot()?;
            let dir = path.join(".github");
            std::fs::create_dir_all(&dir)?;
            let out_path = dir.join("copilot-instructions.md");
            std::fs::write(&out_path, &content)?;
            println!("✓ Exported to {}", out_path.display());
            return Ok(());
        }
        _ => return Err(anyhow::anyhow!(
            "Unsupported format: {}. Supported: markdown, json, claude, cursor, copilot", format
        )),
    };

    println!("{}", output);
    Ok(())
}
