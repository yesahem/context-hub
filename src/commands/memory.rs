use anyhow::Result;
use std::path::PathBuf;

use crate::utils::config::Config;

pub fn display_ttl_memory(path: &PathBuf, _config: &Config) -> Result<()> {
    let storage = crate::core::storage::Storage::new(&path.join(".contexthub/context.db"))?;

    let memories = storage.get_ttl_memory()?;

    if memories.is_empty() {
        println!("No TTL memory stored.");
        return Ok(());
    }

    println!("⏱️  TTL Memory ({} entries)\n", memories.len());

    for mem in memories {
        println!("┌─ {} ─", &mem.commit_hash[..7.min(mem.commit_hash.len())]);
        println!("│ {}", mem.content);
        println!("│ Expires: {}", mem.expires_at.format("%Y-%m-%d %H:%M"));
        println!("└─");
        println!();
    }

    Ok(())
}

pub fn clear_ttl_memory(path: &PathBuf, _config: &Config) -> Result<()> {
    let storage = crate::core::storage::Storage::new(&path.join(".contexthub/context.db"))?;

    storage.clear_ttl_memory()?;
    println!("✓ TTL memory cleared");

    Ok(())
}

pub fn set_ttl(path: &PathBuf, config: &mut Config, days: i32) -> Result<()> {
    config.set_ttl_days(days);
    config.save(path)?;
    println!("✓ TTL set to {} days", days);
    Ok(())
}
