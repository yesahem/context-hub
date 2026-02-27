use git2::{DiffOptions, Repository, Sort};
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub parent_hashes: Vec<String>,
}

pub struct GitAnalyzer {
    repo: Repository,
}

impl GitAnalyzer {
    pub fn new(path: &PathBuf) -> anyhow::Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(Self { repo })
    }

    pub fn get_commit_history(&self, limit: usize) -> anyhow::Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
        revwalk.push_head()?;

        let mut commits = Vec::new();
        for (idx, oid) in revwalk.enumerate() {
            if idx >= limit {
                break;
            }

            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            let hash = oid.to_string();
            let short_hash = hash[..7.min(hash.len())].to_string();

            commits.push(CommitInfo {
                hash: hash.clone(),
                short_hash,
                message: commit.message().unwrap_or("").trim().to_string(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
                date: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_else(|| chrono::Utc::now()),
                parent_hashes: commit.parents().map(|p| p.id().to_string()).collect(),
            });
        }

        Ok(commits)
    }

    /// Returns commits in the range (from_commit, to_commit], newest first.
    /// `from_commit` is exclusive (not included), `to_commit` is inclusive.
    pub fn get_commit_range(
        &self,
        from_commit: &str,
        to_commit: &str,
    ) -> anyhow::Result<Vec<CommitInfo>> {
        let from_oid = git2::Oid::from_str(from_commit)?;
        let to_oid = git2::Oid::from_str(to_commit)?;

        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
        revwalk.push(to_oid)?;
        // Hide `from_oid` and all its ancestors — this gives us (from, to]
        revwalk.hide(from_oid)?;

        let mut commits = Vec::new();

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let hash = oid.to_string();
            let short_hash = hash[..7.min(hash.len())].to_string();

            commits.push(CommitInfo {
                hash: hash.clone(),
                short_hash,
                message: commit.message().unwrap_or("").trim().to_string(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
                date: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_else(|| chrono::Utc::now()),
                parent_hashes: commit.parents().map(|p| p.id().to_string()).collect(),
            });
        }

        Ok(commits)
    }

    pub fn get_diff(&self, commit_hash: &str) -> anyhow::Result<String> {
        let oid = git2::Oid::from_str(commit_hash)?;
        let commit = self.repo.find_commit(oid)?;

        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(true);

        let diff =
            self.repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                'U' => "U",
                _ => "",
            };
            diff_text.push_str(prefix);
            if let Ok(content) = std::str::from_utf8(line.content()) {
                diff_text.push_str(content);
            }
            true
        })?;

        Ok(diff_text)
    }

    pub fn get_commit_count(&self) -> anyhow::Result<usize> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        Ok(revwalk.count())
    }

    pub fn get_current_commit_hash(&self) -> anyhow::Result<String> {
        let head = self.repo.head()?;
        let oid = head.target().unwrap();
        Ok(oid.to_string())
    }

    pub fn get_hooks_path(&self) -> PathBuf {
        self.repo.path().join("hooks")
    }

    #[allow(dead_code)]
    pub fn get_workdir(&self) -> Option<PathBuf> {
        self.repo.workdir().map(|p| p.to_path_buf())
    }
}
