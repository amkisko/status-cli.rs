//! Clap CLI definition.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub const LONG_ABOUT: &str = "\
Search the awesome-status catalog and check live status pages from the terminal.

Examples:
  status search github
  status show \"GitHub\"
  status list --limit 20
  status check github
  status watch --from ~/.config/status/watchlist.toml
  status fetch https://www.githubstatus.com

Catalog data is bundled offline. Live checks use public HTTP (no API keys).

Documentation: https://github.com/amkisko/status-cli.rs
Report issues: https://github.com/amkisko/status-cli.rs/issues";

pub const AFTER_HELP: &str = "\
Output:
  -o plain          Human-readable text (default)
  --plain           Script-stable key=value lines
  -o json           Pretty JSON
  --json            Compact JSON for scripts

Exit codes: 0 success, 1 general error, 2 usage, 3 degraded (with --fail-if-degraded), 4 network/fetch, 5 I/O

Automation:
  status check --from ~/.config/status/watchlist.toml --fail-if-degraded --append-jsonl checks.jsonl
  status watch --from ~/.config/status/watchlist.toml

Run `status help <command>` for command-specific examples.";

#[derive(Parser)]
#[command(
    name = "status",
    version,
    author,
    about = "Status page CLI — search catalog and check live status",
    long_about = LONG_ABOUT,
    after_help = AFTER_HELP,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Output format: plain (human-readable) or json.
    #[arg(
        short,
        long,
        global = true,
        default_value = "plain",
        value_enum,
        env = "STATUS_OUTPUT"
    )]
    pub output: OutputFormatArg,

    /// Emit compact JSON (scripts). Overrides --output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Emit script-stable plain text (key=value lines).
    #[arg(long, global = true, conflicts_with = "json")]
    pub plain: bool,

    /// Suppress non-essential stderr output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show debug details on errors.
    #[arg(long, global = true, env = "STATUS_DEBUG")]
    pub debug: bool,

    /// Show extra context on errors.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable color in supported output.
    #[arg(long, global = true, env = "STATUS_NO_COLOR")]
    pub no_color: bool,

    /// HTTP timeout in seconds (default: 10).
    #[arg(long, global = true, env = "STATUS_TIMEOUT")]
    pub timeout: Option<u64>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    Plain,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fuzzy search services in the catalog
    #[command(after_help = "Examples:\n  status search slack\n  status search github --json")]
    Search {
        /// Search query
        query: String,
        /// Maximum results (default: 20)
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// List service names from the catalog
    #[command(after_help = "Examples:\n  status list\n  status list --limit 100")]
    List {
        /// Maximum names to show (default: 50)
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Show catalog details for a service
    #[command(after_help = "Examples:\n  status show GitHub\n  status show openai")]
    Show {
        /// Service name (exact or fuzzy)
        name: String,
    },
    /// Resolve a service (or URL) and fetch live status
    #[command(
        after_help = "Examples:\n  status check github\n  status check https://status.openai.com\n  status check --from ~/.config/status/watchlist.toml --fail-if-degraded"
    )]
    Check {
        /// Service name or status page URL
        #[arg(required_unless_present = "from")]
        target: Option<String>,
        /// Watchlist file (TOML/JSON/line list of names or URLs)
        #[arg(long = "from", value_name = "FILE")]
        from: Option<PathBuf>,
        /// Exit 3 when any result is degraded or missing a status
        #[arg(long)]
        fail_if_degraded: bool,
        /// Append one compact JSON object per result
        #[arg(long = "append-jsonl", value_name = "FILE")]
        append_jsonl: Option<PathBuf>,
        /// Maximum extracted text length
        #[arg(long, default_value_t = 10000)]
        max_length: usize,
    },
    /// Fetch live status from a status page URL
    #[command(
        after_help = "Examples:\n  status fetch https://www.githubstatus.com\n  status fetch https://www.githubstatus.com --fail-if-degraded --append-jsonl checks.jsonl"
    )]
    Fetch {
        /// Status page URL
        url: String,
        /// Exit 3 when the result is degraded or missing a status
        #[arg(long)]
        fail_if_degraded: bool,
        /// Append one compact JSON object for the result
        #[arg(long = "append-jsonl", value_name = "FILE")]
        append_jsonl: Option<PathBuf>,
        /// Maximum extracted text length
        #[arg(long, default_value_t = 10000)]
        max_length: usize,
    },
    /// Interactive watchlist dashboard (terminal UI)
    #[command(
        after_help = "Keys: q/Esc quit · r refresh · j/k or ↑/↓ select\nExamples:\n  status watch github\n  status watch --from ~/.config/status/watchlist.toml --interval 30"
    )]
    Watch {
        /// Service name or status page URL
        #[arg(required_unless_present = "from")]
        target: Option<String>,
        /// Watchlist file (TOML/JSON/line list of names or URLs)
        #[arg(long = "from", value_name = "FILE")]
        from: Option<PathBuf>,
        /// Refresh interval in seconds (default: 30)
        #[arg(long, default_value_t = 30, env = "STATUS_WATCH_INTERVAL")]
        interval: u64,
        /// Maximum extracted text length
        #[arg(long, default_value_t = 10000)]
        max_length: usize,
    },
    /// Print shell completions
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Print a man page to stdout
    Man,
    /// Print version information
    Version,
}
