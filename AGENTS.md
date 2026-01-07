# Repository Guidelines

## Project Structure & Module Organization
The Rust CLI lives under `src/`, with `src/main.rs` as the entry point and command dispatch. CLI definitions are in `src/cli.rs`, Git operations in `src/git.rs`, configuration handling in `src/config.rs`, and the interactive UI in `src/ui.rs`. AI providers are grouped in `src/ai/` (`mod.rs`, `openai.rs`, `anthropic.rs`). Build artifacts go to `target/`. Configuration files may exist at `./.rust-git-cli.toml`, `~/.config/rust-git-cli/config.toml`, or `~/.rust-git-cli.toml`.

## Build, Test, and Development Commands
- `cargo build`: Debug build for local iteration.
- `cargo run -- <command>`: Run the CLI (e.g., `cargo run -- status`, `commit`, `diff`, `log`, `init`).
- `cargo test`: Run all unit tests.
- `cargo fmt`: Format Rust sources (required before commit).
- `cargo clippy -- -D warnings`: Lint with warnings treated as errors.
- `cargo build --release` or `cargo install --path .`: Release build or local install.

## Coding Style & Naming Conventions
Use standard Rust formatting via `cargo fmt` (4-space indentation). Follow Rust naming conventions: `snake_case` for functions/modules, `CamelCase` for types, and lower-case CLI subcommand names. Keep diffs focused and avoid introducing unused dependencies.

## Testing Guidelines
Tests are currently module-local using `#[cfg(test)]` blocks (for example in `src/ai/openai.rs`) and functions named with a `test_` prefix. Add new unit tests close to the code they validate. Run `cargo test` or `cargo test <name>` to target a specific test.

## CI & Release
CI runs on GitHub Actions via `.github/workflows/rust.yml` with a test matrix across Linux/Windows/macOS and Rust stable/beta/nightly (beta excluded on macOS/Windows). Formatting and clippy checks are strict on stable, and docs build with warnings as errors. Tagged releases build and upload platform binaries, and a RustSec audit job runs on Ubuntu.

## Commit & Pull Request Guidelines
Commit messages follow Conventional Commits and are bilingual on one line, e.g., `feat(openai): 新增... Add ...`. Include a scope when it clarifies the area touched. For pull requests, describe the behavior change, link relevant issues, and include the test commands you ran; for CLI behavior changes, add a short sample command/output snippet.

## Configuration & Security
Never commit API keys. Prefer environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`) or local config files. Configuration lookup order is: project `.rust-git-cli.toml` → user `~/.config/rust-git-cli/config.toml` → fallback `~/.rust-git-cli.toml`.
