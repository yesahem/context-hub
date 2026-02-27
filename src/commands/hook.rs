use anyhow::Result;
use std::path::PathBuf;

pub fn install_hook(path: &PathBuf) -> Result<()> {
    let git = crate::core::git::GitAnalyzer::new(path)?;
    let hooks_dir = git.get_hooks_path();

    let hook_content = r#"#!/bin/sh
# ContextHub post-commit hook
# This hook automatically syncs context after each commit

# Check if we're in a ContextHub initialized repo
if [ -d ".contexthub" ]; then
    # Only sync last commit to avoid overwhelming the system
    contexthub sync --last 1 &
fi
"#;

    let hook_path = hooks_dir.join("post-commit");
    std::fs::write(&hook_path, hook_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }

    println!("✓ Git post-commit hook installed");
    println!("  Path: {}", hook_path.display());

    Ok(())
}

pub fn uninstall_hook(path: &PathBuf) -> Result<()> {
    let git = crate::core::git::GitAnalyzer::new(path)?;
    let hooks_dir = git.get_hooks_path();
    let hook_path = hooks_dir.join("post-commit");

    if hook_path.exists() {
        let content = std::fs::read_to_string(&hook_path)?;
        if content.contains("ContextHub") {
            std::fs::remove_file(&hook_path)?;
            println!("✓ Git post-commit hook removed");
        } else {
            println!("⚠️  Hook exists but doesn't belong to ContextHub");
        }
    } else {
        println!("No post-commit hook found");
    }

    Ok(())
}
