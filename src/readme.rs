//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "youtube",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run youtube accounts list or youtube login",
            },
        });
        return;
    }
    println!("{README}");
}

/// The Apify YouTube Scraper actor spec version.
pub const API_VERSION: &str = "v2";

const RULES: &[&str] = &[
    "Pass --account <name>, --api-key <key>, or configure APIFY_TOKEN.",
    "Run 'youtube accounts list' first if you do not know which accounts exist; ask the human which to use.",
    "On code auth_required, stop and surface the remediation string. Do not retry.",
    "YouTube scraper operations run synchronously via Apify and usually take 30–60 seconds.",
    "Results are YAML by default; pass --json when parsing programmatically in agent loops.",
];

const README: &str = r#"# youtube - agent operating manual

A native Rust CLI for searching and scraping YouTube videos, channels, playlists, and streams via
the Apify YouTube Scraper actor. Results are YAML on stdout, errors are YAML on stderr, and `--json`
switches both to JSON. Progress indicators go to stderr, so stdout is always safe to parse.

## Credentials and Accounts

API tokens can be supplied in three ways:
1. Stored account: `--account <name>` (short `-a <name>`)
2. Explicit flag: `--api-key <key>`
3. Environment variable: `APIFY_TOKEN`

### Managing accounts

    youtube login [<name>] [--api-key <key>]  # opens browser to copy Apify API token
    youtube accounts add <name> --api-key <key> [--force]
    printf %s "$KEY" | youtube accounts add <name> --api-key-stdin
    youtube accounts list [--check]
    youtube accounts test <name>
    youtube accounts remove <name> --yes

`add` tests the token against Apify `/v2/users/me` before storing it. The key is securely saved in the OS
keystore (DPAPI on Windows, Keychain on macOS, libsecret on Linux); only account metadata reaches `config.yaml`.

`list` reports `stored` without touching the network. `--check` validates each stored key against Apify
and reports `valid`, `rejected`, or `unreachable`.

`remove` deletes the key locally from this machine.

## Commands

### Search

Search YouTube for videos, shorts, and livestreams matching a query:

    youtube search <QUERY> [OPTIONS]

Options:
    --max-results <N>      Maximum videos to return (default: 10, 0 = unlimited)
    --max-shorts <N>       Maximum Shorts to return (default: 0)
    --max-streams <N>      Maximum streams to return (default: 0)
    --sort <ORDER>         Sort by: relevance, rating, date, views
    --date <FILTER>        Upload date filter: hour, today, week, month, year
    --length <FILTER>      Length filter: under4, between420, plus20
    --type <TYPE>          Video type: video, movie
    --hd                   Only HD videos
    --live                 Only live videos
    --four-k / --4k        Only 4K videos
    --subtitles            Only videos with subtitles/CC
    --cc                   Only Creative Commons videos
    --download-subs        Download subtitles
    --subs-lang <LANG>     Subtitle language (default: en)
    --subs-format <FMT>    Subtitle format: srt, vtt, xml, plaintext (default: srt)
    --auto-subs            Prefer auto-generated subtitles
    --since <DATE>         Only videos published after this date (e.g. 2025-01-01 or '7 days')

### Scrape

Scrape a YouTube URL (video, channel, playlist, or hashtag):

    youtube scrape <URL> [OPTIONS]

Options:
    --max-results <N>      Maximum regular videos (default: 10, 0 = unlimited)
    --max-shorts <N>       Maximum Shorts to return (default: 0)
    --max-streams <N>      Maximum streams to return (default: 0)
    --sort <ORDER>         Sort channel videos by: NEWEST, POPULAR, OLDEST
    --download-subs        Download subtitles
    --subs-lang <LANG>     Subtitle language (default: en)
    --subs-format <FMT>    Subtitle format: srt, vtt, xml, plaintext (default: srt)
    --auto-subs            Prefer auto-generated subtitles
    --since <DATE>         Only videos published after this date (e.g. 2025-01-01 or '7 days')

## Errors

Failures print YAML (or JSON) on stderr with a stable `code`, and exit with a matching status:

    0  ok
    1  error          unclassified - report it and stop
    2  network        retry once, then stop
    3  auth_required  stop; give the human the `remediation` string verbatim
    4  not_found      resource or actor does not exist; do not retry
    5  rate_limited   back off before trying again
    6  invalid_input  fix the call
    7  no_account     run `youtube accounts list` or `youtube login`

`detail` carries the HTTP status and API response body. An envelope carries `remediation` when there
is a specific command that fixes the problem."#;
