# Youtube CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Youtube-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Youtube-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Youtube-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Youtube-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Youtube-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A blazing fast, native command-line tool and agent interface for searching and scraping YouTube videos, channels, playlists, and streams via [Apify](https://apify.com). Built in native Rust 2024 for developers and autonomous AI agents.

---

## Highlights

- ⚡ **Sub-3ms Startup**: Compiled as a native static binary with zero runtime dependencies. Cold-starts in ~1–3 ms.
- 🔐 **OS Keystore Integration**: `youtube login` stores API tokens securely in native OS vaults (macOS Keychain, Linux Secret Service / Keyutils, Windows DPAPI).
- 🎬 **Comprehensive Scraping**: Scrape individual videos, channels, playlists, hashtags, and search queries with full subtitle download support.
- 🤖 **AI Agent Native**: Structured YAML by default for terminal readability, `--json` for tool loops and `jq`, structured error envelopes, and built-in `youtube agent-readme`.
- 🛡️ **Flexible Credentials**: Support for named account profiles (`--account <name>`), per-command flag (`--api-key <key>`), and `APIFY_TOKEN` environment variable.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Youtube-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Youtube-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`youtube-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Youtube-Cli/releases/download/v1.0.0/youtube-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`youtube-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Youtube-Cli/releases/download/v1.0.0/youtube-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`youtube-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Youtube-Cli/releases/download/v1.0.0/youtube-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`youtube-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/Youtube-Cli/releases/download/v1.0.0/youtube-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Quickstart

### 1. Authenticate

Get your API token from [Apify](https://console.apify.com/account/integrations) and store it in your OS keystore:

```bash
# Interactive login (stores key securely in Keychain / DPAPI / libsecret)
youtube login

# Named account login
youtube login work

# Headless / CI login via stdin
echo "$APIFY_TOKEN" | youtube login ci --api-key-stdin

# Or use environment variable directly
export APIFY_TOKEN=your-token-here
```

### 2. Search YouTube

```bash
# Basic video search
youtube search "rust programming tutorial"

# Limit results and include Shorts / Streams
youtube search "cooking recipes" --max-results 5 --max-shorts 3 --max-streams 2

# Sort and date filtering
youtube search "tech news" --sort date --date today

# Filter 4K, HD, and download subtitles
youtube search "conference keynote" --4k --download-subs --subs-lang en

# Return raw JSON for jq or agent tool calling
youtube search "ai agents" --json
```

### 3. Scrape Videos, Channels, and Playlists

```bash
# Scrape video metadata and details
youtube scrape "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Scrape latest 10 videos from a channel
youtube scrape "https://www.youtube.com/@Fireship" --max-results 10

# Channel videos sorted by popularity
youtube scrape "https://www.youtube.com/@Fireship" --sort POPULAR --max-results 5

# Scrape an entire playlist
youtube scrape "https://www.youtube.com/playlist?list=PLObrtcm1Kw6PmbXg8bmfJN-o2Hgx8sidf"

# Download subtitles as SRT
youtube scrape "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --download-subs --subs-format srt
```

---

## Command Reference

### Commands

| Command | Description |
|:---|:---|
| `youtube search <QUERY>` | Search YouTube for videos, shorts, and live streams |
| `youtube scrape <URL>` | Scrape a YouTube URL (video, channel, playlist, or hashtag) |
| `youtube login [NAME]` | Authenticate with Apify and store token in the OS keystore |
| `youtube accounts list` | List configured accounts (pass `--check` to verify tokens with Apify) |
| `youtube accounts test <NAME>` | Test connectivity and identity for a stored account |
| `youtube accounts add <NAME>` | Add a named account to the keystore |
| `youtube accounts remove <NAME>`| Remove a named account and purge its token from the keystore |
| `youtube agent-readme` | Print embedded agent manual (pass `--json` for machine-readable rules) |

### Common Options

| Flag | Description | Default |
|:---|:---|:---|
| `-a, --account <NAME>` | Stored account profile to use | None |
| `--api-key <KEY>` | Explicit Apify API token | None |
| `--json` | Output raw JSON instead of YAML | `false` |
| `--max-results <N>` | Maximum videos to return (0 = unlimited) | `10` |
| `--max-shorts <N>` | Maximum Shorts to return | `0` |
| `--max-streams <N>` | Maximum streams to return | `0` |
| `--sort <ORDER>` | Search: `relevance, rating, date, views`; Channel: `NEWEST, POPULAR, OLDEST` | None |
| `--date <FILTER>` | Upload date filter: `hour, today, week, month, year` | None |
| `--length <FILTER>` | Duration filter: `under4, between420, plus20` | None |
| `--hd` | Filter for high-definition videos | `false` |
| `--four-k`, `--4k` | Filter for 4K resolution videos | `false` |
| `--download-subs` | Download video subtitles | `false` |
| `--subs-lang <LANG>` | Subtitle language (`en`, `es`, `de`, `fr`, etc.) | `en` |
| `--subs-format <FMT>` | Subtitle format (`srt`, `vtt`, `xml`, `plaintext`) | `srt` |
| `--since <DATE>` | Filter videos published after date (e.g. `2025-01-01` or `7 days`) | None |

---

## AI Agent Integration

`youtube` is built from the ground up for LLM agents:

- **Self-documenting**: Run `youtube agent-readme --json` to get rules, capabilities, and exit codes.
- **Structured Error Envelopes**: Errors return JSON/YAML on `stderr` with stable exit codes:
  - `0`: Success
  - `1`: Unclassified error
  - `2`: Network error (retry once)
  - `3`: `auth_required` (stop, prompt user)
  - `4`: `not_found`
  - `5`: `rate_limited` (back off)
  - `6`: `invalid_input`
  - `7`: `no_account`

---

## License

MIT License. Copyright (c) 2026 SpaceCorps. Originally authored by Niels Bosma (`nielsbosma/Youtube.Console`).
