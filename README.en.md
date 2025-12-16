# Rust Git CLI

An intelligent Git commit tool that generates bilingual (Chinese/English) commit messages using AI.

## Features

- **AI-Powered** - Supports OpenAI, Anthropic, and custom endpoints (e.g., DeepSeek)
- **Bilingual Commits** - Automatically generates Chinese/English commit messages following Conventional Commits
- **Smart Staging** - Detects unstaged changes and prompts for confirmation
- **Interactive UI** - Colored output, diff preview, commit confirmation
- **Flexible Configuration** - Multi-level config files and environment variables

## Installation

```bash
# Build from source
git clone https://github.com/duanyongcheng/rust-git-cli.git
cd rust-git-cli
cargo build --release

# Install to system
cargo install --path .
```

**Requirements**: Rust 1.70+, Git 2.0+

## Quick Start

### 1. Initialize Configuration

```bash
rust-git-cli init                 # Create global config (~/.config/rust-git-cli/config.toml)
rust-git-cli init --local         # Create project config (.rust-git-cli.toml)
```

### 2. Set API Key

```bash
# Recommended: Use environment variables
export OPENAI_API_KEY="your-api-key"
# or
export ANTHROPIC_API_KEY="your-api-key"
```

### 3. Usage

```bash
rust-git-cli                      # Check repository status (default)
rust-git-cli commit               # Generate AI commit message
rust-git-cli commit --show-diff   # Preview diff before generation
rust-git-cli commit --debug       # Debug mode
```

## Commands

| Command | Description |
|---------|-------------|
| `status` | Check repository status (default) |
| `commit` | Generate and execute AI commit |
| `diff` | Show code changes |
| `log` | Show commit history |
| `init` | Initialize config file |

### commit Options

```bash
rust-git-cli commit [OPTIONS]

Options:
  --api-key <KEY>      Specify API key temporarily
  --model <MODEL>      Specify AI model (e.g., gpt-4, deepseek-v3)
  --base-url <URL>     Custom API endpoint
  --auto               Skip confirmation and commit directly
  --show-diff          Preview diff before generation
  --debug              Show raw AI response
```

### log Options

```bash
rust-git-cli log [OPTIONS]

Options:
  -n, --count <N>      Number of commits to show (default: 10)
  --grep <PATTERN>     Filter by content
  --author <NAME>      Filter by author
  --since <DATE>       Start date (e.g., "2024-01-01" or "1 week ago")
  --until <DATE>       End date
  --full               Show full commit message
```

### diff Options

```bash
rust-git-cli diff [OPTIONS]

Options:
  --staged             Show only staged changes
```

## Configuration

Config file lookup order:
1. `./.rust-git-cli.toml` (project-level)
2. `~/.config/rust-git-cli/config.toml` (user-level)
3. `~/.rust-git-cli.toml` (user-level fallback)

### Example Configuration

```toml
[ai]
provider = "openai"                      # openai or anthropic
model = "gpt-4"                          # Model name
api_key_env = "OPENAI_API_KEY"           # API key environment variable name
# api_key = "sk-..."                     # Direct setting (not recommended)
# base_url = "https://api.deepseek.com/v1"  # Custom endpoint
max_tokens = 2000

[commit]
max_diff_size = 4000                     # Max diff characters sent to AI
auto_stage = false                       # Auto-stage all changes
```

### API Key Priority

1. Command line argument `--api-key`
2. Config file `api_key`
3. Environment variable (specified by `api_key_env`)
4. Interactive input

## Commit Message Format

Generated commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
feat(auth): 添加用户认证功能
Add user authentication feature

实现了JWT令牌验证
Implement JWT token validation
添加了用户登录接口
Add user login endpoint
```

### Commit Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation |
| `style` | Code formatting |
| `refactor` | Code refactoring |
| `test` | Testing |
| `chore` | Build/tooling |
| `perf` | Performance |

## Workflow

```bash
# 1. Check status
$ rust-git-cli status

# 2. View changes
$ rust-git-cli diff

# 3. Generate commit
$ rust-git-cli commit

# When unstaged changes are detected:
# Unstaged changes detected:
# ──────────────────────────────────────────────────
#   M src/main.rs
#   ? src/new_file.rs
# ──────────────────────────────────────────────────
# Do you want to stage all changes (git add .)? (Y/n)

# After AI generation, choose action:
# - Accept and commit: Accept and commit
# - Edit message: Edit before commit
# - Regenerate: Generate again
# - Cancel: Cancel
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| API connection failed | Check network, verify API key, use `--debug` for details |
| JSON parsing error | Use `--debug` to view raw response, try different model |
| Config not working | Check file path and TOML format |
| Commit failed | Ensure Git user is configured (`git config user.name/email`) |

```bash
# Debug mode
rust-git-cli commit --debug

# View help
rust-git-cli --help
rust-git-cli commit --help
```

## Development

```bash
cargo build                       # Build
cargo test                        # Test
cargo fmt                         # Format
cargo clippy -- -D warnings       # Lint
```

### Project Structure

```
src/
├── main.rs          # Entry point and command dispatch
├── cli.rs           # CLI definitions (clap)
├── config.rs        # Configuration management
├── git.rs           # Git operations (git2)
├── ui.rs            # Interactive UI (dialoguer)
└── ai/
    ├── mod.rs       # AI client abstraction
    ├── openai.rs    # OpenAI implementation
    └── anthropic.rs # Anthropic implementation
```

## License

MIT License

## Credits

- [git2-rs](https://github.com/rust-lang/git2-rs) - Git operations
- [clap](https://github.com/clap-rs/clap) - Command line parsing
- [dialoguer](https://github.com/console-rs/dialoguer) - Interactive UI
- [colored](https://github.com/colored-rs/colored) - Colored output
