use anyhow::Result;
use std::path::PathBuf;

use crate::utils::config::Config;

pub fn show_config(config: &Config) -> Result<()> {
    println!("📋 Configuration\n");
    println!("Ollama:");
    println!("  Endpoint:  {}", config.ollama.endpoint);
    println!("  Model:     {}", config.ollama.model);
    println!("  Temperature: {}", config.ollama.temperature);
    println!();
    println!("Context:");
    println!(
        "  Default commit range: {}",
        config.context.default_commit_range
    );
    println!(
        "  Max tokens/commit:     {}",
        config.context.max_tokens_per_commit
    );
    println!("  TTL days:              {}", config.context.ttl_days);
    println!();
    println!("Git:");
    println!("  Auto sync:    {}", config.git.auto_sync);
    println!("  Hook enabled: {}", config.git.hook_enabled);

    Ok(())
}

pub fn set_config_model(path: &PathBuf, config: &mut Config, model: String) -> Result<()> {
    config.set_model(model.clone());
    config.save(path)?;
    println!("✓ Model set to: {}", model);
    Ok(())
}

pub fn set_config_ollama_url(path: &PathBuf, config: &mut Config, url: String) -> Result<()> {
    config.set_ollama_url(url.clone());
    config.save(path)?;
    println!("✓ Ollama URL set to: {}", url);
    Ok(())
}
