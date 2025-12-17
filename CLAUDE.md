# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Rust-based Git commit tool that generates bilingual (Chinese/English) commit messages using AI providers (OpenAI/Anthropic). Uses Conventional Commits format with automatic repository detection and interactive prompts.

## Build and Development Commands

```bash
# Development
cargo build                       # Debug build
cargo run -- status               # Check repository status (default command)
cargo run -- diff                 # Show all changes
cargo run -- diff --staged        # Show staged changes only
cargo run -- commit               # Generate AI commit message
cargo run -- commit --debug       # Debug mode - shows AI raw response
cargo run -- log -n 20            # Show commit history (interactive, supports AI changelog)
cargo run -- init                 # Initialize config (~/.config/rust-git-cli/)
cargo run -- init --local         # Initialize config in current directory

# Quality checks (required before commit)
cargo fmt                         # Format code
cargo clippy -- -D warnings       # Lint with warnings as errors
cargo test                        # Run all tests

# Release
cargo build --release
cargo install --path .
```

## Architecture

### Command Flow
```
CLI (src/cli.rs) → Main dispatch (src/main.rs) → Command handlers
                                                      ↓
                   Git ops (src/git.rs) ← AI generation (src/ai/) → UI (src/ui.rs)
                                                      ↓
                                          Config (src/config.rs)
```

### Key Components

**`src/main.rs`** - Entry point with command routing
- `init` command bypasses git repo check (line 24)
- `check_and_stage_changes()` prompts before staging unstaged files
- Commands: Status (default), Commit, Diff, Log, Init

**`src/ai/mod.rs`** - AI client abstraction
- `AIClient` enum dispatches to OpenAI/Anthropic providers
- `CommitMessage` struct with custom deserializers for flexible JSON parsing
- `build_prompt()` generates bilingual prompt with diff context
- `generate_changelog()` creates AI summaries from selected commits

**`src/git.rs`** - Git operations via git2 crate
- `GitRepo` wrapper with `get_status()`, `get_diff()`, `get_combined_diff()`, `get_branch_info()`, `get_commits()`
- Handles unborn branches (new repos without commits)

**`src/config.rs`** - TOML configuration
- Lookup order: `./.rust-git-cli.toml` → `~/.config/rust-git-cli/config.toml` → `~/.rust-git-cli.toml`
- API key priority: CLI arg → config file → env var → interactive prompt

**`src/ui.rs`** - Interactive prompts via dialoguer
- `CommitUI::confirm_commit()` shows Accept/Edit/Regenerate/Cancel options
- Color-coded diff preview

### Data Structures

```rust
// src/ai/mod.rs - Bilingual commit message
CommitMessage {
    commit_type: String,      // feat, fix, docs, etc.
    scope: Option<String>,
    description: String,      // Chinese
    description_en: String,   // English
    body: Option<Vec<String>>,    // Chinese details
    body_en: Option<Vec<String>>, // English details
    breaking_change: Option<String>,
}

// src/cli.rs - Commands enum with clap derive
Commands::Status | Commit { api_key, model, base_url, auto, show_diff, debug } | Diff { staged } | Log { count, grep, author, since, until, full, api_key, model, base_url, debug } | Init { local, force }
```

## Configuration

Config file example (`~/.config/rust-git-cli/config.toml`):
```toml
[ai]
provider = "openai"           # or "anthropic"
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.deepseek.com/v1"  # optional, for proxies
max_tokens = 2000

[commit]
max_diff_size = 4000
auto_stage = false
```

## Debugging

```bash
cargo run -- commit --debug   # Shows raw AI response and JSON parsing
```

## CI/CD

GitHub Actions (`.github/workflows/rust.yml`):
- Test matrix: Linux/Windows/macOS × stable/beta/nightly Rust
- Quality gates: `cargo fmt --check`, `cargo clippy -- -D warnings`
- Release builds on tag push for multiple platforms
- Security audit via `rustsec/audit-check`