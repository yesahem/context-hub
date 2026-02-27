# ContextHub — Architecture & Changelog

> Auto-generated architecture document reflecting the current state of the codebase after the refactoring pass.

## Overview

ContextHub is a CLI tool that creates a **centralized context memory** for AI coding assistants. It extracts structured context from git commits using a local LLM (Ollama), stores it in SQLite, and exports it in formats consumable by Claude, Cursor, GitHub Copilot, and others.

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   CLI Layer  │───▶│  Core Layer  │───▶│   Storage    │
│  (commands/) │    │  (core/)     │    │  (SQLite)    │
└──────────────┘    └──────┬───────┘    └──────────────┘
                           │
                    ┌──────▼───────┐
                    │  Ollama LLM  │
                    │  (local)     │
                    └──────────────┘
```

- - -

## Directory Structure

```
context-hub/
├── Cargo.toml              # Project manifest (edition 2021)
├── src/
│   ├── main.rs             # CLI entry point (clap derive)
│   ├── commands/           # User-facing command handlers
│   │   ├── init.rs         # Initialize .contexthub/ in a repo
│   │   ├── sync.rs         # Extract context from commits
│   │   ├── context.rs      # Display & export stored context
│   │   ├── memory.rs       # TTL memory management
│   │   ├── config_cmd.rs   # Configuration show/set
│   │   ├── doctor.rs       # System health check
│   │   └── hook.rs         # Git post-commit hook install/uninstall
│   ├── core/               # Business logic
│   │   ├── context.rs      # ContextProcessor — orchestrates git+llm+storage
│   │   ├── git.rs          # GitAnalyzer — git2 wrapper
│   │   ├── llm.rs          # LlmProcessor — Ollama HTTP client
│   │   └── storage.rs      # Storage — SQLite CRUD
│   ├── utils/              # Configuration & logging
│   │   ├── config.rs       # JSON config (de)serialization
│   │   └── logger.rs       # File-based logging (env_logger)
│   └── ui/                 # TUI layer (ratatui) — future feature, dead code
│       ├── mod.rs
│       ├── components/
│       └── screens/
├── ARCHITECTURE.md          # This file
├── REVIEW.md                # Code review findings
└── SPEC.md                  # Original specification
```

- - -

## Architecture Changes (Refactoring Changelog)

### Phase 1 — Foundation Fixes

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **Edition fix** | `Cargo.toml` | Changed `edition = "2024"` → `"2021"` (2024 doesn't exist) |
| **Init rewrite** | `commands/init.rs` | Now checks git FIRST before creating dirs, actually creates SQLite DB via `Storage::new()`, adds `.contexthub/` to `.gitignore` |
| **Binary name** | `hook.rs`, `doctor.rs`, `init.rs`, `context.rs` | All user-facing strings now say `contexthub` instead of `ctxhub` |
| **Doctor formatting** | `commands/doctor.rs` | Fixed `println!("✓ Git: ",)` trailing comma bug, added dynamic recommendation numbering |
| **Unused imports** | `core/context.rs`, `core/llm.rs`, `commands/memory.rs`, `ui/components/widgets.rs` | Removed `Arc`, `PathBuf`, `ContextProcessor`, `Style` |
| **Unused deps** | `Cargo.toml` | Removed `clap_complete`, `thiserror`, `directories`, `indicatif`, `winapi` |

### Phase 2 — Critical Bug Fixes

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **Git commit range** | `core/git.rs` | Rewrote `get_commit_range()` — was collecting commits OLDER than `from` (inverted logic). Now uses `revwalk.hide(from_oid)` to correctly return `(from, to]` range |
| **Commit dedup** | `core/storage.rs` | Added `has_commit()` method to check if a commit is already stored |
| **Replace curl** | `core/llm.rs` | `is_ollama_running()` now uses `reqwest::blocking::get()` instead of shelling out to `curl` |
| **WAL mode** | `core/storage.rs` | Added `PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;` on connection open for safe concurrent access |
| **Remove is\_repo** | `core/git.rs` | Removed `is_repo()` which always returned `true` — constructor already validates via `Repository::discover()` |

### Phase 3 — Core Feature: Incremental Context Chaining

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **Full ExtractedContext storage** | `core/context.rs`, `core/storage.rs` | `llm_extracted_context` column now stores the full `ExtractedContext` JSON (summary, files\_changed, key\_details, technologies, impact) instead of just the summary string |
| **Incremental chaining** | `core/llm.rs`, `core/context.rs` | `extract_context()` now accepts `previous_context: Option<&str>` and includes it in the LLM prompt so each commit builds on the previous one's understanding |
| **Latest context fetch** | `core/storage.rs` | Added `get_latest_context_summary()` to retrieve the last stored summary for chaining |
| **Oldest-first processing** | `commands/sync.rs` | Commits are reversed before processing so context builds chronologically forward |
| **store\_global\_context signature** | `core/storage.rs` | Added `llm_extracted_json: &str` parameter (4th arg) to store the full JSON separately from the summary |

### Phase 4 — Safety: Token Estimation & Diff Truncation

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **Token estimation** | `core/context.rs` | Added `chars / 4` heuristic to estimate token count. Diffs exceeding `max_tokens_per_commit` (configurable, default 1000) are truncated with a notice |

### Phase 5 — Operational Robustness

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **Init guard** | `main.rs` | Added `require_init()` check before Sync, Context, Memory, Config, Hook, Status commands. Init and Doctor remain unguarded |
| **TTL cleanup** | `main.rs` | `cleanup_expired_ttl()` is now called automatically before every `sync` command |
| **Sync dedup + progress** | `commands/sync.rs` | Skips already-processed commits (via `has_commit`), shows skip count, cleaner progress output |

### Phase 6 — Export Format Templates

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **CLAUDE.md export** | `core/context.rs`, `commands/context.rs` | `contexthub context --export claude` generates a `CLAUDE.md` file with project overview, recent changes, and technologies |
| **Cursor export** | `core/context.rs`, `commands/context.rs` | `contexthub context --export cursor` generates `.cursorrules` |
| **Copilot export** | `core/context.rs`, `commands/context.rs` | `contexthub context --export copilot` generates `.github/copilot-instructions.md` |
| **Helper methods** | `core/context.rs` | Added `build_project_summary()`, `extract_technologies()` to aggregate context data for exports |

### Phase 7 — Warning Cleanup

| Change | File(s) | Description |
| ------ | ------- | ----------- |
| **Dead code suppression** | Multiple files | Added `#[allow(dead_code)]` on intentionally-public API methods (logger, LLM setters, git helpers) and TUI module |
| **Unused variable prefixes** | `commands/memory.rs` | Prefixed unused params with `_` |

- - -

## Data Flow

### Sync Flow (most important)

```
contexthub sync --last 10
    │
    ▼
1. require_init() — check .contexthub/ exists
2. cleanup_expired_ttl() — remove stale TTL entries
3. Get commits from git (via git2)
4. Reverse to process oldest-first
5. Dedup: skip commits already in DB (has_commit)
6. Check Ollama is running (reqwest blocking GET)
7. For each commit:
   a. Get diff (git2)
   b. Estimate tokens, truncate if needed
   c. Fetch previous context summary from DB
   d. Send to Ollama with incremental prompt
   e. Parse JSON response → ExtractedContext
   f. Store summary + full JSON in global_context table
   g. Store summary in ttl_memory table
```

### Export Flow

```
contexthub context --export claude
    │
    ▼
1. Fetch all global_context rows
2. Aggregate file paths, technologies
3. Build structured markdown
4. Write to CLAUDE.md / .cursorrules / .github/copilot-instructions.md
```

- - -

## Database Schema

``` sql
-- Permanent context storage
CREATE TABLE global_context (
    id INTEGER PRIMARY KEY,
    commit_hash TEXT UNIQUE NOT NULL,
    commit_message TEXT,
    commit_date TEXT,           -- RFC 3339
    context_summary TEXT,       -- Human-readable summary
    files_changed TEXT,         -- JSON array of file paths
    llm_extracted_context TEXT, -- Full ExtractedContext JSON
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Temporary context (auto-expires)
CREATE TABLE ttl_memory (
    id INTEGER PRIMARY KEY,
    commit_hash TEXT NOT NULL,
    content TEXT,
    expires_at TEXT,            -- RFC 3339
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

**Pragmas**: `journal_mode=WAL`, `busy_timeout=5000`

- - -

## Configuration

Stored in `.contexthub/config.json`:

``` json
{
  "ollama": {
    "endpoint": "http://localhost:11434",
    "model": "llama3.2",
    "temperature": 0.3,
    "max_tokens": 2048
  },
  "context": {
    "default_commit_range": 10,
    "max_tokens_per_commit": 1000,
    "global_retention_days": -1,
    "ttl_days": 7
  },
  "git": {
    "auto_sync": false,
    "hook_enabled": false
  },
  "ui": {
    "theme": "tokyo-night"
  }
}
```

- - -

## CLI Commands

| Command | Description |
| ------- | ----------- |
| `contexthub init` | Initialize `.contexthub/` in current git repo |
| `contexthub sync [--last N] [--from HASH]` | Extract context from commits via Ollama |
| `contexthub context [--export FORMAT]` | Display or export context (md, json, claude, cursor, copilot) |
| `contexthub memory ttl [--clear] [--set-ttl DAYS]` | Manage TTL memory |
| `contexthub config show` | Show current configuration |
| `contexthub config set-model MODEL` | Change Ollama model |
| `contexthub config set-ollama-url URL` | Change Ollama endpoint |
| `contexthub hook install/uninstall` | Manage post-commit git hook |
| `contexthub doctor` | System health check |
| `contexthub status` | Show sync status |

- - -

## Key Dependencies

| Crate | Version | Purpose |
| ----- | ------- | ------- |
| clap | 4.5 | CLI argument parsing (derive) |
| git2 | 0.19 | Git repository access (libgit2 binding) |
| rusqlite | 0.32 | SQLite database (bundled) |
| reqwest | 0.12 | HTTP client for Ollama API (json + blocking) |
| tokio | 1.40 | Async runtime |
| ratatui | 0.28 | Terminal UI framework (future use) |
| crossterm | 0.28 | Terminal backend |
| chrono | 0.4 | Date/time handling |
| serde | 1.0 | Serialization framework |
| anyhow | 1.0 | Error handling |

- - -

## Future Work

* **TUI Integration**: The `ui/` module is fully written but not wired into any commands. Needs terminal setup/teardown and async channel integration.
* **Unit Tests**: No tests exist yet. Priority areas: storage CRUD, git range logic, LLM response parsing, init flow.
* **Global Retention**: `global_retention_days` config exists but no cleanup is implemented (TTL cleanup is wired).
* **Shell Completions**: `clap_complete` was removed; could be re-added as an optional feature.