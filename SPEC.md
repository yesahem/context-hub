# Global Context Storage - Project Specification

## Project Overview

**Project Name:** ContextHub (working title)

**Core Functionality:** A CLI tool that creates a centralized context memory for coding assistants, extracting and storing repository context from git commits locally, making it shareable across all AI coding tools.

**Target Users:** Developers who use multiple AI coding assistants (Claude Code, Cursor, VS Code AI, Kira, OpenCode) and want consistent context sharing without re-explaining codebase history.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      ContextHub CLI                         │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   TUI Layer  │  │ Git Analyzer │  │  LLM Processor   │  │
│  │  (ratatui)   │  │  (git2-rs)   │  │   (ollama)       │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                   │             │
│         └─────────────────┼───────────────────┘             │
│                           ▼                                  │
│                  ┌────────────────┐                         │
│                  │  Context Store │                         │
│                  │  (.contexthub) │                         │
│                  └────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Feature Specifications

### 1. Initialization (`<app_name> init`)

**Behavior:**
- User runs `<app_name> init` in repository root
- Creates `.contexthub/` directory (similar to `.git/`)
- Initializes configuration file `config.json`
- Sets up local SQLite database for context storage
- Detects if already initialized and prompts accordingly

**Directory Structure:**
```
.contexthub/
├── config.json          # Configuration (llm model, settings)
├── context.db          # SQLite database for context storage
├── memory/
│   ├── ttl/            # TTL-based short-term memory
│   └── global/        # Long-term global context
└── cache/             # Cached LLM responses
```

### 2. Git Commit Context Extraction

**Flow:**
1. **Detect new commits** - Watch for git commit events via:
   - Manual trigger: `<app_name> sync`
   - Git hook integration (optional post-commit hook)
   
2. **Commit selection UI** - Show interactive TUI with:
   - List of recent commits (paginated, 20 per page)
   - Commit hash, message, date, author
   - Option to select range (from commit X to HEAD)
   - Option to select "all commits" with warning about token limits
   
3. **Diff extraction** - For each commit in range:
   - Get `git diff` between commit and its parent
   - Extract changed files, added/removed lines
   - Capture commit message as context header

4. **Token management**:
   - Default: Extract context from last 10 commits
   - User can specify: `--from-commit <hash>` or `--last N commits`
   - Warn if estimated tokens > LLM context window (adjustable, default 8K)

### 3. Local LLM Integration (Ollama)

**Requirements:**
- Detect if Ollama is installed and running
- Support configurable model selection
- Default model: `llama3.2` or user-specified
- Support custom Ollama endpoint (default: `http://localhost:11434`)

**Processing Pipeline:**
```
For each commit in selected range:
  1. Get commit metadata (hash, message, author, date)
  2. Get git diff for that commit
  3. Construct prompt with:
     - Commit message as task description
     - File changes summary
     - Full diff content
  4. Send to Ollama with custom system prompt
  5. Parse LLM response into structured context
  6. Store in SQLite with commit hash reference
```

**System Prompt Template:**
```
You are a code context analyzer. Given a git commit diff, extract:
1. What feature/fix was implemented
2. Key files changed and their purpose
3. Technical details worth remembering
4. Dependencies or relationships with other parts

Respond in JSON format:
{
  "summary": "brief description",
  "files_changed": ["file1", "file2"],
  "key_details": ["detail1", "detail2"],
  "technologies": ["React", "PostgreSQL"],
  "impact": "high|medium|low"
}
```

### 4. Context Storage (`.contexthub/`)

**Database Schema (SQLite):**

```sql
-- Global context (persistent across sessions)
CREATE TABLE global_context (
    id INTEGER PRIMARY KEY,
    commit_hash TEXT UNIQUE NOT NULL,
    commit_message TEXT,
    commit_date DATETIME,
    context_summary TEXT,
    files_changed TEXT, -- JSON array
    llm_extracted_context TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- TTL-based memory (temporary, expires after N days)
CREATE TABLE ttl_memory (
    id INTEGER PRIMARY KEY,
    commit_hash TEXT NOT NULL,
    content TEXT,
    expires_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index for efficient lookups
CREATE INDEX idx_global_commit ON global_context(commit_hash);
CREATE INDEX idx_global_date ON global_context(commit_date);
CREATE INDEX idx_ttl_expires ON ttl_memory(expires_at);
```

### 5. Two Memory Types

#### Global Context Memory
- **Purpose:** Long-term repository memory
- **Storage:** SQLite `global_context` table
- **Retention:** Forever (or user-configured)
- **Use case:** When starting new session with any AI tool
- **Extraction:** Process all commits from initial commit to HEAD

#### TTL Memory
- **Purpose:** Short-term working context
- **Storage:** SQLite `ttl_memory` table  
- **Retention:** Configurable (default: 7 days)
- **Use case:** Recent work context that expires
- **Extraction:** Process last N commits (default: 3)

### 6. CLI Commands

| Command | Description |
|---------|-------------|
| `<app> init` | Initialize context storage in current repo |
| `<app> sync` | Sync/extract context from recent commits |
| `<app> sync --from <commit>` | Sync from specific commit to HEAD |
| `<app> sync --last N` | Sync last N commits |
| `<app> context` | Display current global context |
| `<app> context --export` | Export context to file (JSON/Markdown) |
| `<app> context --format markdown` | Export as markdown for AI tools |
| `<app> memory ttl` | Show TTL memory contents |
| `<app> memory ttl --clear` | Clear all TTL memory |
| `<app> memory ttl --set-ttl DAYS` | Set TTL duration |
| `<app> config` | Show/set configuration |
| `<app> config --set-model MODEL` | Set LLM model |
| `<app> config --set-ollama-url URL` | Set Ollama endpoint |
| `<app> status` | Show sync status, last commit processed |
| `<app> hook install` | Install git post-commit hook |
| `<app> hook uninstall` | Remove git hook |
| `<app> doctor` | Check Ollama, git, permissions |

### 7. TUI Design

**Color Palette (Sexy & Classy):**
- **Background:** `#1a1b26` (Tokyo Night dark)
- **Primary:** `#7aa2f7` (Blue)
- **Secondary:** `#bb9af7` (Purple)
- **Accent:** `#9ece6a` (Green - success)
- **Warning:** `#e0af68` (Orange)
- **Error:** `#f7768e` (Red/Pink)
- **Text:** `#c0caf5` (Light blue-gray)
- **Muted:** `#565f89` (Gray-blue)

**TUI Screens:**

1. **Welcome/Init Screen**
   - Logo/ASCII art
   - "Initializing ContextHub..."
   - Progress indicators
   - Success/failure feedback

2. **Sync Screen**
   - Commit list with selection
   - Progress bar for processing
   - Live output from LLM
   - Commit-by-commit status

3. **Context Display**
   - Formatted context output
   - Copy to clipboard option
   - Export options

4. **Configuration**
   - Model selection dropdown
   - Ollama URL input
   - TTL settings

**Interactions:**
- Arrow keys for navigation
- Enter to select
- Space for multi-select
- Tab to switch panels
- Esc to go back
- Ctrl+C to exit

---

## Technical Implementation

### Technology Stack

**Language:** Rust (for performance, safety, and excellent CLI/TUI libraries)

**Key Dependencies:**
- `ratatui` - TUI framework
- `git2` - Git operations
- `rusqlite` - SQLite database
- `reqwest` - HTTP client for Ollama
- `serde` / `serde_json` - Serialization
- `tokio` - Async runtime
- `chrono` - Date/time handling

### Key Modules

```
src/
├── main.rs              # CLI entry, argument parsing
├── commands/
│   ├── init.rs          # Initialize command
│   ├── sync.rs          # Sync/extract context
│   ├── context.rs       # Display/export context
│   ├── memory.rs        # TTL memory management
│   ├── config.rs        # Configuration management
│   └── doctor.rs        # Health checks
├── core/
│   ├── git.rs           # Git operations (commits, diffs)
│   ├── llm.rs           # Ollama integration
│   ├── context.rs       # Context extraction logic
│   └── storage.rs       # SQLite operations
├── ui/
│   ├── mod.rs           # TUI orchestration
│   ├── screens/
│   │   ├── init.rs       # Init screen
│   │   ├── sync.rs      # Sync screen
│   │   └── context.rs   # Context display
│   └── components/      # Reusable UI components
└── utils/
    ├── config.rs        # Config file handling
    └── logger.rs        # Logging setup
```

### Git Hook Integration

Create `.git/hooks/post-commit`:
```bash
#!/bin/sh
# ContextHub post-commit hook
# Triggers context sync after each commit
contexthub sync --last 1
```

---

## User Flows

### Flow 1: First Time Setup

```bash
# 1. User initializes repository
$ ctxhub init

# TUI shows:
# ┌────────────────────────────────────────┐
# │     ContextHub Initialization         │
# │                                        │
# │  ✓ Detected git repository             │
# │  ✓ Creating .contexthub/ directory    │
# │  ✓ Initializing SQLite database       │
# │  ✓ Configuration created              │
# │                                        │
# │  Would you like to extract context    │
# │  from your commit history?             │
# │                                        │
# │  [Yes, extract now]  [No, skip]        │
# └────────────────────────────────────────┘
```

### Flow 2: Context Sync

```bash
# User syncs context
$ ctxhub sync

# TUI shows commit selection
# ┌────────────────────────────────────────┐
# │  Select Commits to Process             │
# │                                        │
# │  ○ Select all (warning: may be large)  │
# │  ◉ Select range                        │
# │  ○ Last N commits                      │
# │                                        │
# │  Commits (newest first):               │
# │  ○ a1b2c3d Fix authentication bug     │
# │  ● d4e5f6 Add user dashboard           │
# │  ○ g7h8i9 Refactor API endpoints       │
# │  ○ j1k2l3 Initial commit               │
# │                                        │
# │  [Process Selected] [Cancel]           │
# └────────────────────────────────────────┘
```

### Flow 3: Export Context

```bash
# Export for AI coding assistant
$ ctxhub context --format markdown

# Outputs:
# ## Repository Context
#
# ### Recent Changes
# - **a1b2c3d**: Fixed authentication bug in login flow
#   - Modified: `auth/login.ts`, `middleware/auth.ts`
#   - Technologies: JWT, Express
#
# - **d4e5f6**: Added user dashboard with analytics
#   - Modified: `components/Dashboard.tsx`
#   - Technologies: React, Chart.js
# ...
```

---

## Configuration

### Default Config (`~/.contexthub/config.json` or `.contexthub/config.json`)

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
    "global_retention_days": -1, // -1 = forever
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

---

## Acceptance Criteria

1. **Initialization**: Running `ctxhub init` creates `.contexthub/` with all required files
2. **Git Integration**: Successfully reads commit history and extracts diffs
3. **LLM Processing**: Connects to local Ollama and extracts meaningful context
4. **Storage**: Context stored in SQLite, retrievable by commit hash
5. **TTL Memory**: Memory expires after configured days
6. **TUI**: All screens render correctly with specified color scheme
7. **Export**: Context exports in valid JSON/Markdown format
8. **Git Hook**: Post-commit hook triggers sync (when enabled)
9. **No Auth**: All data stored locally, no network calls except to local Ollama
10. **Cross-Platform**: Works on macOS, Linux, Windows

---

## Future Enhancements (Post-MVP)

- Multiple LLM provider support (llama.cpp, LM Studio)
- Context search/retrieval by keyword
- Context merging for branches
- Integration with specific AI tools (Cursor, Claude Code)
- Web interface for context browsing
- Context versioning
