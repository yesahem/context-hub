# ContextHub

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/rust-stable-green" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-orange" alt="License">
</p>

> **Global Context Storage for AI Coding Assistants**

ContextHub solves the problem of constantly re-explaining your codebase to different AI coding tools (Claude Code, Cursor, VS Code AI, Kira, OpenCode, etc.). It extracts and stores repository context from git commits locally, making it instantly available to any coding assistant.

## Why ContextHub?

- **No more repeating yourself** - Each AI tool needs context about your codebase
- **Local only** - All data stored on your machine, no cloud, no auth
- **Local LLM** - Uses Ollama for context extraction (privacy-first)
- **Git-native** - Works with your existing workflow

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Commands](#commands)
4. [Configuration](#configuration)
5. [Ollama Setup](#ollama-setup)
6. [Architecture](#architecture)
7. [Git Hook Integration](#git-hook-integration)
8. [Exporting Context](#exporting-context)
9. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

- **Rust** - Install from [rustup.rs](https://rustup.rs/)
- **Ollama** - Install from [ollama.ai](https://ollama.ai/)
- **Git** - Required for commit history access

### Build from Source

```bash
# Clone or navigate to the project
cd context-hub

# Build the binary
cargo build --release

# Add to your PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

Or for development:
```bash
cargo build
./target/debug/contexthub --help
```

---

## Quick Start

### 1. Initialize a Repository

```bash
cd your-project
contexthub init
```

This creates `.contexthub/` directory with:
```
.contexthub/
├── config.json          # Your configuration
├── context.db          # SQLite database
├── memory/
│   ├── ttl/           # Short-term memory
│   └── global/        # Long-term context
└── cache/            # LLM response cache
```

### 2. Configure Your Model

```bash
# Check available models in Ollama
curl http://localhost:11434/api/tags

# Set your preferred model
contexthub config set-model llama3.2
# or
contexthub config set-model kimi-k2.5:cloud
```

### 3. Extract Context

```bash
# Sync last 10 commits (default)
contexthub sync

# Sync last N commits
contexthub sync --last 5

# Sync from specific commit to HEAD
contexthub sync --from abc1234
```

### 4. View Context

```bash
# View stored context in terminal
contexthub context

# Export as Markdown (for AI tools)
contexthub context --export markdown

# Export as JSON
contexthub context --export json
```

---

## Commands

### `contexthub init`

Initialize ContextHub in the current directory.

```bash
contexthub init [--path /path/to/repo]
```

Creates `.contexthub/` directory with config and database.

---

### `contexthub sync`

Extract and store context from git commits using LLM.

```bash
contexthub sync [OPTIONS]

OPTIONS:
  --path <PATH>          Path to repository (default: current directory)
  --from <COMMIT>        Process from specific commit hash to HEAD
  --last <N>             Process last N commits
```

**Examples:**
```bash
# Sync last 10 commits (default)
contexthub sync

# Sync last 3 commits
contexthub sync --last 3

# Sync from a specific commit
contexthub sync --from a1b2c3d

# Process specific repository
contexthub sync --path ~/projects/myapp
```

---

### `contexthub context`

Display stored repository context.

```bash
contexthub context [OPTIONS]

OPTIONS:
  --path <PATH>        Path to repository
  --export <FORMAT>    Export format: markdown, json
```

**Examples:**
```bash
# View context in terminal
contexthub context

# Export for Claude Code/Cursor
contexthub context --export markdown > context.md

# Export as JSON
contexthub context --export json > context.json
```

---

### `contexthub memory`

Manage TTL (short-term) memory.

```bash
contexthub memory <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
  ttl         Manage TTL memory

OPTIONS:
  --path <PATH>        Path to repository
  --clear             Clear all TTL memory
  --set-ttl <DAYS>    Set TTL expiration days
```

**Examples:**
```bash
# View TTL memory
contexthub memory ttl

# Clear all TTL memory
contexthub memory ttl --clear

# Set TTL to 14 days
contexthub memory ttl --set-ttl 14
```

---

### `contexthub config`

Manage configuration settings.

```bash
contexthub config <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
  show           Show current configuration
  set-model      Set Ollama model
  set-ollama-url Set Ollama endpoint

OPTIONS:
  --path <PATH>    Path to repository
```

**Examples:**
```bash
# View current config
contexthub config show

# Set model
contexthub config set-model llama3.2

# Set custom Ollama URL
contexthub config set-ollama-url http://localhost:11434
```

---

### `contexthub hook`

Manage git hooks for auto-sync.

```bash
contexthub hook <COMMAND> [OPTIONS]

COMMANDS:
  install      Install post-commit hook
  uninstall    Remove post-commit hook

OPTIONS:
  --path <PATH>    Path to repository
```

**Examples:**
```bash
# Auto-sync after every commit
contexthub hook install

# Remove auto-sync
contexthub hook uninstall
```

---

### `contexthub doctor`

System health check - verifies dependencies and configuration.

```bash
contexthub doctor [--path /path/to/repo]
```

Checks:
- Git repository status
- Ollama installation
- Ollama running status
- ContextHub initialization
- Database existence

---

### `contexthub status`

Show sync status and statistics.

```bash
contexthub status [--path /path/to/repo]
```

Shows:
- Total commits in repository
- Stored context entries
- Last processed commit
- Ollama connection status

---

## Configuration

### Default Config

Created at `.contexthub/config.json` on init:

```json
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

### Configuration Options

| Option | Type | Description |
|--------|------|-------------|
| `ollama.endpoint` | string | Ollama API URL |
| `ollama.model` | string | Model to use for extraction |
| `ollama.temperature` | float | LLM temperature (0.0-1.0) |
| `ollama.max_tokens` | int | Max tokens per response |
| `context.default_commit_range` | int | Default commits to sync |
| `context.max_tokens_per_commit` | int | Token budget per commit |
| `context.global_retention_days` | int | Global context retention (-1 = forever) |
| `context.ttl_days` | int | TTL memory expiration days |
| `git.auto_sync` | bool | Auto-sync on commit |
| `git.hook_enabled` | bool | Hook installed status |

---

## Ollama Setup

### Installation

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Windows
# Download from https://ollama.ai
```

### Start Ollama

```bash
# Start the server
ollama serve

# In another terminal, list available models
ollama list

# Pull a model
ollama pull llama3.2
```

### Recommended Models

For context extraction, these models work well:

| Model | Size | Best For |
|-------|------|----------|
| `llama3.2` | 2GB | General purpose, fast |
| `qwen3-coder:480b` | 480MB | Code-specific, lightweight |
| `kimi-k2.5:cloud` | ~340MB | Cloud model, requires internet |
| `codellama` | 4GB | Code-focused |

### Check Ollama Status

```bash
# From ContextHub
contexthub doctor

# Or directly
curl http://localhost:11434/api/tags
```

---

## Architecture

### How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                     ContextHub CLI                          │
├─────────────────────────────────────────────────────────────┤
│  1. User runs: ctxhub sync --last 10                       │
│                                                             │
│  2. Git Analyzer reads commit history                       │
│     - Gets commit metadata                                  │
│     - Extracts git diffs                                    │
│                                                             │
│  3. LLM Processor sends to Ollama                           │
│     - Constructs prompt with commit + diff                  │
│     - Extracts structured context                           │
│                                                             │
│  4. Storage saves to SQLite                                  │
│     - Global context (permanent)                            │
│     - TTL memory (temporary, expires)                       │
└─────────────────────────────────────────────────────────────┘
```

### Two Memory Types

#### Global Context (Long-term)
- Stored in `global_context` table
- Never expires by default
- Contains: commit hash, message, date, LLM summary, files changed
- Used for: Initial context to any AI tool

#### TTL Memory (Short-term)
- Stored in `ttl_memory` table
- Expires after configured days (default: 7)
- Contains: Recent commit context
- Used for: Current working context

### Directory Structure

```
your-repo/
├── .git/                  # Git repository
├── .contexthub/           # ContextHub data
│   ├── config.json        # Configuration
│   ├── context.db        # SQLite database
│   ├── memory/
│   │   ├── ttl/         # TTL memory (optional)
│   │   └── global/      # Global context (optional)
│   ├── cache/           # LLM response cache
│   └── logs/            # Application logs
├── src/                  # Your code
├── package.json          # Dependencies
└── ...
```

---

## Git Hook Integration

### Automatic Sync

Install the post-commit hook to automatically sync context after each commit:

```bash
contexthub hook install
```

This creates `.git/hooks/post-commit`:
```bash
#!/bin/sh
# ContextHub post-commit hook
if [ -d ".contexthub" ]; then
    exec ctxhub sync --last 1
fi
```

### Manual Sync

Or sync manually when needed:

```bash
# After a significant commit
contexthub sync --last 1

# Before starting a new coding session
contexthub sync --last 5
```

---

## Exporting Context

### For Claude Code / Cursor

Export as markdown and paste into the AI:

```bash
contexthub context --export markdown
```

Output:
```markdown
# Repository Context

## Recent Changes

### abc1234: Add user authentication
- **Date:** 2024-01-15
- **Summary:** Implemented JWT-based authentication with login/logout
- **Files:** auth/login.ts, auth/middleware.ts, config/jwt.ts

### def5678: Create dashboard component
- **Date:** 2024-01-14
- **Summary:** Added main dashboard with charts and user stats
- **Files:** components/Dashboard.tsx, hooks/useStats.ts
```

### For API Integration

Export as JSON for programmatic access:

```bash
contexthub context --export json
```

---

## Troubleshooting

### Ollama Not Running

```bash
# Start Ollama
ollama serve

# Verify
curl http://localhost:11434/api/tags
```

### Model Not Found

```bash
# List available models
ollama list

# Pull a model
ollama pull llama3.2

# Update config
contexthub config set-model llama3.2
```

### No Commits Found

Make sure you're in a git repository with commits:
```bash
git log --oneline
```

### Context Not Extracting

Check the model is compatible:
```bash
# Test directly
curl -X POST http://localhost:11434/api/generate \
  -d '{"model": "llama3.2", "prompt": "hello", "stream": false}'
```

### Database Issues

Delete and reinitialize:
```bash
rm -rf .contexthub
contexthub init
```

---

## Best Practices

1. **Install the hook** - `contexthub hook install` for automatic syncing
2. **Use appropriate model** - `qwen3-coder` for code-heavy repos
3. **Regular sync** - Run `contexthub sync` before starting new features
4. **Export context** - Before switching AI tools, export current context
5. **TTL settings** - Adjust `ttl_days` based on your workflow

---

## License

MIT License - See [LICENSE](LICENSE) for details.

---

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.

### Development

```bash
# Clone the repo
git clone https://github.com/yourusername/context-hub.git
cd context-hub

# Run in development
cargo run -- --help

# Run tests
cargo test

# Build
cargo build --release
```
