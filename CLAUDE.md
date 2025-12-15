# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Rust-based Git commit tool that generates intelligent, bilingual (Chinese/English) commit messages using AI providers (OpenAI/Anthropic/DeepSeek). Features automatic repository detection, diff visualization, and structured commit message generation following Conventional Commits specification.

## Build and Development Commands

```bash
# Build the project
cargo build --release

# Run in development mode
cargo run

# Run with specific command
cargo run -- status              # Check repository status (default)
cargo run -- diff                # Show all changes
cargo run -- diff --staged        # Show staged changes only
cargo run -- commit              # Generate AI commit message
cargo run -- commit --show-diff  # Show diff preview before generation
cargo run -- commit --debug      # Debug mode - shows AI raw response
cargo run -- init                # Initialize config file in ~/.config/rust-git-cli/
cargo run -- init --local        # Initialize config file in current directory

# Run specific tests and checks
cargo test                        # Run all tests
cargo test test_commit_message    # Run specific test

# Code quality
rustup component add clippy       # Install clippy if missing
rustup component add rustfmt      # Install rustfmt if missing
cargo fmt                         # Format all code
cargo clippy -- -D warnings       # Run linter with warnings as errors
cargo clippy --fix                # Auto-fix clippy warnings
cargo fix --allow-dirty --allow-staged  # Auto-fix compiler warnings

# Build for distribution
cargo build --release               # Production optimized build
target/release/rust-git-cli --help   # Test release binary

# Install globally
cargo install --path .             # Install from local source
cargo install rust-git-cli          # Install from crates.io (if published)

# Development workflow example
make test                          # Run all tests, fmt, and clippy
make check                        # Check syntax without building
```

## High-Level Architecture

### Command Flow Pipeline
1. **CLI Parsing** (`src/cli.rs:7-58`) → Parses command-line arguments using clap with derive macros
2. **Main Dispatch** (`src/main.rs:44-73`) → Routes to appropriate command handler using match on Commands enum
3. **Git Operations** (`src/git.rs`) → Interacts with git repository via git2 crate
4. **AI Generation** (`src/ai/mod.rs`) → Sends prompts to AI providers (OpenAI/Anthropic/DeepSeek) via HTTP
5. **User Interface** (`src/ui.rs`) → Interactive prompts with dialoguer crate, color-coded output
6. **Configuration** (`src/config.rs`) → TOML config files with hierarchical lookup strategy

### Key Data Structures
- `AIClient` enum (`src/ai/mod.rs`) - Provider dispatch pattern
- `CommitMessage` struct (`src/ai/mod.rs`) - Bilingual commit messages
- `GitRepo` wrapper (`src/git.rs`) - Safe git operations
- `Args` struct (`src/cli.rs`) - Command-line argument parsing

### Module Responsibilities

**`src/main.rs`**
- Entry point with command orchestration
- Special handling: `init` command bypasses git repo check
- Command handlers: `handle_status_command`, `handle_diff_command`, `handle_commit_command`, `handle_init_command`
- Enhanced `execute_commit`: Now prompts user before staging unstaged changes

**`src/git.rs`** 
- `GitRepo` wrapper around `git2::Repository`
- Key methods: `get_status()`, `get_diff()`, `get_combined_diff()`, `get_branch_info()`
- Edge case handling: unborn branches (new repos without commits)

**`src/ai/mod.rs`**
- `AIClient` enum dispatches to providers (enum pattern due to async trait limitations)
- `CommitMessage` struct with bilingual support (Chinese/English)
- `build_prompt()` generates bilingual commit request
- Custom deserializers handle various response formats

**`src/ai/openai.rs` & `src/ai/anthropic.rs`**
- Provider-specific HTTP clients
- Debug mode support for troubleshooting API responses
- Flexible parsing: handles both string and array body formats
- `breaking_change` field accepts boolean or string values

**`src/cli.rs`**
- Command definitions with clap derive macros
- Commands: Status, Commit (with --debug flag), Diff, Init
- Model parameter is optional (falls back to config)

**`src/config.rs`**
- Config file lookup order: `./.rust-git-cli.toml` → `~/.config/rust-git-cli/config.toml` → `~/.rust-git-cli.toml`
- API key resolution: CLI arg → config file → env var → interactive prompt
- Supports custom base URLs for API proxies or alternative endpoints

**`src/ui.rs`**
- `CommitUI` struct provides all user interaction methods
- Interactive prompts using `dialoguer` crate
- Color-coded diff preview with line limits
- Commit action selection (Accept/Edit/Regenerate/Cancel)
- API key input handling (Note: currently shows plaintext)

## Bilingual Commit Message Format

The tool generates commit messages in Chinese/English bilingual format:

```
type(scope): 中文简要描述
English brief description

中文详细说明第一点
English explanation point 1
中文详细说明第二点
English explanation point 2
```

### CommitMessage Structure
- `description`: Chinese summary
- `description_en`: English translation
- `body`: Array of Chinese explanations
- `body_en`: Array of English translations
- Flexible deserialization handles both legacy string and new array formats

## Configuration

### API Key Setup Priority
1. Command-line: `--api-key YOUR_KEY`
2. Config file: `api_key = "your-key"` in `.rust-git-cli.toml`
3. Environment: `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`
4. Interactive prompt (fallback)

### Config File Fields
- `api_key_env`: Name of environment variable to check (e.g., "OPENAI_API_KEY")
- `api_key`: Direct API key (use `api_key_env` for security)
- `base_url`: Custom API endpoint (for proxies/alternative services)
- `model`: AI model to use (can be overridden with --model flag)

## Debug Features

Use `--debug` flag to troubleshoot AI responses:
```bash
rust-git-cli commit --debug
```

Shows:
- Raw HTTP response from API
- Extracted message content before JSON parsing
- Helps identify format mismatches or API issues

## Error Handling Patterns

- `anyhow::Result` for error propagation with context
- Special handling for `UnbornBranch` in new repositories
- Debug mode displays full API responses for troubleshooting
- Null-safe parsing for optional API response fields

## Commit Generation Flow

1. Check repository status (`GitRepo::get_status()`)
2. Generate combined diff (staged + unstaged changes)
3. Build commit context (branch, file count, line changes)
4. Send bilingual prompt to AI provider
5. Parse response with flexible deserializers
6. Format as bilingual commit message
7. Present interactive options (Accept/Edit/Regenerate/Cancel)
8. **NEW**: Check for unstaged changes and prompt user to stage them
9. Execute commit via `git commit -m` (with optional `git add .` based on user choice)

## CI/CD Pipeline

### GitHub Actions Workflow (`.github/workflows/rust.yml`)
- **Test Matrix**: Builds and tests on Linux, Windows, macOS with Rust stable/beta/nightly
- **Quality Gates**: Format check (cargo fmt), linting (cargo clippy with warnings as errors)
- **Release Builds**: Automated release binaries for multiple platforms on tag push
- **Security Audit**: Automatic dependency vulnerability scanning
- **MSRV**: Implicit minimum supported Rust version based on dependencies

## Known Issues and Considerations

### Potential Bugs to Address
1. **Path handling**: Config path resolution may fail when `home_dir()` returns `None`
2. **JSON parsing**: Anthropic client's simple string slicing for JSON extraction is fragile
3. **Security**: API keys entered via stdin are shown in plaintext
4. **HTTP timeouts**: No timeout configured for API requests
5. **Error messages**: API errors may expose sensitive information
6. **File permissions**: Config files containing API keys lack proper permission settings

### Testing Infrastructure
- Unit tests in `src/` directory (use `cargo test` to run)
- Integration tests in `tests/` directory (if created later)
- CI builds for all supported platforms
- Example: `cargo test test_commit_message` for specific test module

### Development Workflow
1. Run `cargo fmt && cargo clippy -- -D warnings` before committing
2. Use `cargo test` to ensure all tests pass
3. Use `cargo run -- commit --debug` to troubleshoot AI issues
4. Test cross-platform with `cargo build --release` on target systems

### Best Practices
- Always use environment variables for API keys instead of config files
- Review generated commit messages before accepting
- Use `--debug` flag when encountering API issues (`cargo run -- commit --debug`)
- Keep diff size reasonable (default max: 4000 chars) for better AI responses
- Monitor AI provider rate limits and adjust diff size accordingly