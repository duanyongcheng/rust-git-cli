use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, help = "Path to check (default: current directory)")]
    pub path: Option<PathBuf>,

    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check repository status (default)
    Status,

    /// Generate commit message using AI
    Commit {
        #[arg(long, help = "API key for AI service (or set OPENAI_API_KEY env var)")]
        api_key: Option<String>,

        #[arg(long, help = "AI model to use (overrides config)")]
        model: Option<String>,

        #[arg(long, help = "Custom API base URL (e.g., https://api.openai.com/v1)")]
        base_url: Option<String>,

        #[arg(long, help = "Auto-commit without confirmation")]
        auto: bool,

        #[arg(long, help = "Show diff in the prompt")]
        show_diff: bool,

        #[arg(long, help = "Debug mode - show AI raw response")]
        debug: bool,
    },

    /// Show git diff
    Diff {
        #[arg(long, help = "Show staged changes only")]
        staged: bool,
    },

    /// Initialize configuration file
    Init {
        #[arg(long, help = "Create config in current directory instead of home")]
        local: bool,

        #[arg(long, help = "Force overwrite existing config")]
        force: bool,
    },

    /// Show git commit log (changelog)
    Log {
        #[arg(
            short = 'n',
            long,
            default_value = "10",
            help = "Number of commits to show"
        )]
        count: usize,

        #[arg(long, help = "Show only commits containing this text")]
        grep: Option<String>,

        #[arg(long, help = "Show commits by specific author")]
        author: Option<String>,

        #[arg(
            long,
            help = "Show commits since date (e.g., '2024-01-01' or '1 week ago')"
        )]
        since: Option<String>,

        #[arg(long, help = "Show commits until date")]
        until: Option<String>,

        #[arg(long, help = "Show full commit message")]
        full: bool,

        #[arg(long, help = "API key for AI service (or set OPENAI_API_KEY env var)")]
        api_key: Option<String>,

        #[arg(long, help = "AI model to use (overrides config)")]
        model: Option<String>,

        #[arg(long, help = "Custom API base URL")]
        base_url: Option<String>,

        #[arg(long, help = "Debug mode - show AI raw response")]
        debug: bool,
    },
}
