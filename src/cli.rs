//! Command-line argument hierarchy parsed by `clap`.

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "youtube",
    version,
    about = "High-performance CLI for YouTube searching and scraping via Apify",
    propagate_version = true
)]
pub struct Cli {
    /// Emit raw JSON instead of YAML
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search YouTube for videos, shorts, and streams
    Search(SearchArgs),

    /// Scrape a YouTube URL (video, channel, playlist, or hashtag)
    Scrape(ScrapeArgs),

    /// Authenticate with Apify and store token in OS keystore
    Login(LoginArgs),

    /// Manage named Apify credentials
    Accounts(AccountsArgs),

    /// Print machine-readable agent instructions and rules
    AgentReadme,
}

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Account to use (from stored accounts)
    #[arg(short = 'a', long)]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Maximum videos to return (0 = unlimited)
    #[arg(long, default_value_t = 10)]
    pub max_results: u32,

    /// Maximum Shorts to return
    #[arg(long, default_value_t = 0)]
    pub max_shorts: u32,

    /// Maximum streams to return
    #[arg(long, default_value_t = 0)]
    pub max_streams: u32,

    /// Sort by: relevance, rating, date, views
    #[arg(long)]
    pub sort: Option<String>,

    /// Upload date filter: hour, today, week, month, year
    #[arg(long)]
    pub date: Option<String>,

    /// Length filter: under4, between420, plus20
    #[arg(long)]
    pub length: Option<String>,

    /// Video type: video, movie
    #[arg(long, id = "type")]
    pub video_type: Option<String>,

    /// Only HD videos
    #[arg(long)]
    pub hd: bool,

    /// Only live videos
    #[arg(long)]
    pub live: bool,

    /// Only 4K videos
    #[arg(long, alias = "4k")]
    pub four_k: bool,

    /// Only videos with subtitles/CC
    #[arg(long)]
    pub subtitles: bool,

    /// Only Creative Commons videos
    #[arg(long)]
    pub cc: bool,

    /// Download subtitles
    #[arg(long)]
    pub download_subs: bool,

    /// Subtitle language: any, en, de, es, fr, it, ja, ko, nl, pt, ru
    #[arg(long, default_value = "en")]
    pub subs_lang: String,

    /// Subtitle format: srt, vtt, xml, plaintext
    #[arg(long, default_value = "srt")]
    pub subs_format: String,

    /// Prefer auto-generated subtitles
    #[arg(long)]
    pub auto_subs: bool,

    /// Only videos published after this date (e.g. 2025-01-01 or '7 days')
    #[arg(long)]
    pub since: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ScrapeArgs {
    /// YouTube URL (video, channel, playlist, hashtag, or search results page)
    pub url: String,

    /// Account to use (from stored accounts)
    #[arg(short = 'a', long)]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Maximum regular videos (0 = unlimited)
    #[arg(long, default_value_t = 10)]
    pub max_results: u32,

    /// Maximum Shorts to return
    #[arg(long, default_value_t = 0)]
    pub max_shorts: u32,

    /// Maximum streams to return
    #[arg(long, default_value_t = 0)]
    pub max_streams: u32,

    /// Sort channel videos by: NEWEST, POPULAR, OLDEST
    #[arg(long)]
    pub sort: Option<String>,

    /// Download subtitles
    #[arg(long)]
    pub download_subs: bool,

    /// Subtitle language: any, en, de, es, fr, it, ja, ko, nl, pt, ru
    #[arg(long, default_value = "en")]
    pub subs_lang: String,

    /// Subtitle format: srt, vtt, xml, plaintext
    #[arg(long, default_value = "srt")]
    pub subs_format: String,

    /// Prefer auto-generated subtitles
    #[arg(long)]
    pub auto_subs: bool,

    /// Only videos published after this date (e.g. 2025-01-01 or '7 days')
    #[arg(long)]
    pub since: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// Apify API token (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API key from stdin, e.g. `pbpaste | youtube login --api-key-stdin`
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not attempt to open a browser window for login
    #[arg(long)]
    pub no_browser: bool,

    /// Overwrite existing account credentials
    #[arg(long)]
    pub force: bool,

    /// Skip testing the token against Apify before storing
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AccountsArgs {
    #[command(subcommand)]
    pub command: AccountsSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AccountsSubcommand {
    /// Add or update an account credential in the OS keystore
    Add {
        /// Account name
        name: String,

        /// Apify API token
        #[arg(long, conflicts_with = "api_key_stdin")]
        api_key: Option<String>,

        /// Read token from stdin
        #[arg(long)]
        api_key_stdin: bool,

        /// Overwrite if the account already exists
        #[arg(long)]
        force: bool,

        /// Skip verifying the key against Apify before storing
        #[arg(long)]
        no_verify: bool,
    },

    /// List configured accounts
    List {
        /// Probe Apify API to check validity of each stored key
        #[arg(long)]
        check: bool,
    },

    /// Test a stored account's token against Apify
    Test {
        /// Account name
        name: String,
    },

    /// Remove an account from this machine
    Remove {
        /// Account name
        name: String,

        /// Do not prompt for confirmation
        #[arg(long)]
        yes: bool,
    },
}
