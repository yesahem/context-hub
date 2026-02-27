use std::path::PathBuf;

use crate::core::git::{CommitInfo, GitAnalyzer};
use crate::core::llm::{ExtractedContext, LlmProcessor};
use crate::core::storage::{Storage, GlobalContext};
use crate::utils::config::Config;

pub struct ContextProcessor {
    pub git: GitAnalyzer,
    llm: LlmProcessor,
    storage: Storage,
    config: Config,
}

impl ContextProcessor {
    pub fn new(repo_path: &PathBuf, config: Config) -> anyhow::Result<Self> {
        let git = GitAnalyzer::new(repo_path)?;
        let storage = Storage::new(&repo_path.join(".contexthub/context.db"))?;
        let llm = LlmProcessor::new(config.ollama.clone());
        
        Ok(Self {
            git,
            llm,
            storage,
            config,
        })
    }

    pub fn get_commits(&self, limit: usize) -> anyhow::Result<Vec<CommitInfo>> {
        self.git.get_commit_history(limit)
    }

    pub fn get_commit_range(&self, from: &str, to: &str) -> anyhow::Result<Vec<CommitInfo>> {
        self.git.get_commit_range(from, to)
    }

    pub async fn process_commit(&self, commit: &CommitInfo) -> anyhow::Result<ExtractedContext> {
        let diff = self.git.get_diff(&commit.hash)?;
        
        let files: Vec<String> = diff
            .lines()
            .filter(|l| l.starts_with("+++ b/") || l.starts_with("--- a/"))
            .map(|l| l.replace("+++ b/", "").replace("--- a/", ""))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let context = self.llm
            .extract_context(&commit.message, &diff, &files)
            .await?;

        self.storage.store_global_context(
            commit,
            &context.summary,
            &files,
        )?;

        self.storage.store_ttl_memory(
            &commit.hash,
            &context.summary,
            self.config.context.ttl_days,
        )?;

        Ok(context)
    }

    pub fn get_global_context(&self) -> anyhow::Result<Vec<GlobalContext>> {
        self.storage.get_global_context()
    }

    pub fn get_global_context_since(&self, commit_hash: &str) -> anyhow::Result<Vec<GlobalContext>> {
        self.storage.get_global_context_since(commit_hash)
    }

    pub fn export_context_markdown(&self) -> anyhow::Result<String> {
        let contexts = self.storage.get_global_context()?;
        
        let mut output = String::from("# Repository Context\n\n");
        output.push_str("## Recent Changes\n\n");
        
        for ctx in contexts.iter().take(20) {
            output.push_str(&format!("### {}: {}\n", 
                &ctx.commit_hash[..7.min(ctx.commit_hash.len())],
                ctx.commit_message.lines().next().unwrap_or("No message")
            ));
            output.push_str(&format!("- **Date:** {}\n", ctx.commit_date.format("%Y-%m-%d")));
            output.push_str(&format!("- **Summary:** {}\n", ctx.context_summary));
            
            if !ctx.files_changed.is_empty() {
                let files: Vec<String> = serde_json::from_str(&ctx.files_changed)
                    .unwrap_or_default();
                output.push_str(&format!("- **Files:** {}\n", files.join(", ")));
            }
            output.push('\n');
        }
        
        Ok(output)
    }

    pub fn export_context_json(&self) -> anyhow::Result<String> {
        let contexts = self.storage.get_global_context()?;
        let json = serde_json::to_string_pretty(&contexts)?;
        Ok(json)
    }

    pub fn is_ollama_running(&self) -> bool {
        self.llm.is_ollama_running()
    }

    pub fn get_last_commit(&self) -> anyhow::Result<Option<String>> {
        self.storage.get_last_processed_commit()
    }

    pub fn get_context_count(&self) -> anyhow::Result<usize> {
        self.storage.get_context_count()
    }
}
