use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::git::CommitInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalContext {
    pub id: i64,
    pub commit_hash: String,
    pub commit_message: String,
    pub commit_date: DateTime<Utc>,
    pub context_summary: String,
    pub files_changed: String,
    pub llm_extracted_context: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TtlMemory {
    pub id: i64,
    pub commit_hash: String,
    pub content: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new(db_path: &PathBuf) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        let storage = Self { conn };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS global_context (
                id INTEGER PRIMARY KEY,
                commit_hash TEXT UNIQUE NOT NULL,
                commit_message TEXT,
                commit_date TEXT,
                context_summary TEXT,
                files_changed TEXT,
                llm_extracted_context TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS ttl_memory (
                id INTEGER PRIMARY KEY,
                commit_hash TEXT NOT NULL,
                content TEXT,
                expires_at TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_global_commit ON global_context(commit_hash)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_global_date ON global_context(commit_date)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ttl_expires ON ttl_memory(expires_at)",
            [],
        )?;

        Ok(())
    }

    /// Check if a commit has already been processed (for dedup)
    pub fn has_commit(&self, commit_hash: &str) -> anyhow::Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM global_context WHERE commit_hash = ?1",
            [commit_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn store_global_context(
        &self,
        commit: &CommitInfo,
        context_summary: &str,
        files_changed: &[String],
        llm_extracted_json: &str,
    ) -> anyhow::Result<()> {
        let files_json = serde_json::to_string(files_changed)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO global_context 
             (commit_hash, commit_message, commit_date, context_summary, files_changed, llm_extracted_context)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                commit.hash,
                commit.message,
                commit.date.to_rfc3339(),
                context_summary,
                files_json,
                llm_extracted_json,
            ],
        )?;

        Ok(())
    }

    /// Get the most recently stored context summary for incremental chaining
    pub fn get_latest_context_summary(&self) -> anyhow::Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT context_summary FROM global_context ORDER BY commit_date DESC LIMIT 1",
        )?;
        let result = stmt.query_row([], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn get_global_context(&self) -> anyhow::Result<Vec<GlobalContext>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, commit_hash, commit_message, commit_date, context_summary, 
                    files_changed, llm_extracted_context, created_at
             FROM global_context ORDER BY commit_date DESC",
        )?;

        let contexts = stmt
            .query_map([], |row| {
                Ok(GlobalContext {
                    id: row.get(0)?,
                    commit_hash: row.get(1)?,
                    commit_message: row.get(2)?,
                    commit_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    context_summary: row.get(4)?,
                    files_changed: row.get(5)?,
                    llm_extracted_context: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contexts)
    }

    #[allow(dead_code)]
    pub fn get_global_context_since(
        &self,
        commit_hash: &str,
    ) -> anyhow::Result<Vec<GlobalContext>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, commit_hash, commit_message, commit_date, context_summary, 
                    files_changed, llm_extracted_context, created_at
             FROM global_context 
             WHERE commit_hash = ?1 OR commit_date >= (
                 SELECT commit_date FROM global_context WHERE commit_hash = ?1
             )
             ORDER BY commit_date DESC",
        )?;

        let contexts = stmt
            .query_map([commit_hash], |row| {
                Ok(GlobalContext {
                    id: row.get(0)?,
                    commit_hash: row.get(1)?,
                    commit_message: row.get(2)?,
                    commit_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    context_summary: row.get(4)?,
                    files_changed: row.get(5)?,
                    llm_extracted_context: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contexts)
    }

    pub fn get_last_processed_commit(&self) -> anyhow::Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT commit_hash FROM global_context ORDER BY commit_date DESC LIMIT 1")?;

        let result = stmt.query_row([], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn store_ttl_memory(
        &self,
        commit_hash: &str,
        content: &str,
        ttl_days: i32,
    ) -> anyhow::Result<()> {
        let expires_at = Utc::now() + Duration::days(ttl_days as i64);

        self.conn.execute(
            "INSERT INTO ttl_memory (commit_hash, content, expires_at) VALUES (?1, ?2, ?3)",
            params![commit_hash, content, expires_at.to_rfc3339()],
        )?;

        Ok(())
    }

    pub fn get_ttl_memory(&self) -> anyhow::Result<Vec<TtlMemory>> {
        let now = Utc::now().to_rfc3339();

        let mut stmt = self.conn.prepare(
            "SELECT id, commit_hash, content, expires_at, created_at
             FROM ttl_memory 
             WHERE expires_at > ?1
             ORDER BY created_at DESC",
        )?;

        let memories = stmt
            .query_map([now], |row| {
                Ok(TtlMemory {
                    id: row.get(0)?,
                    commit_hash: row.get(1)?,
                    content: row.get(2)?,
                    expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    pub fn clear_ttl_memory(&self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM ttl_memory", [])?;
        Ok(())
    }

    pub fn cleanup_expired_ttl(&self) -> anyhow::Result<usize> {
        let now = Utc::now().to_rfc3339();
        let deleted = self
            .conn
            .execute("DELETE FROM ttl_memory WHERE expires_at <= ?1", [now])?;
        Ok(deleted)
    }

    pub fn get_context_count(&self) -> anyhow::Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM global_context", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}
