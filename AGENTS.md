# AGENTS.md

Notes for whoever extends this next.

`youtube` is a native Rust CLI for searching and scraping YouTube videos, channels, playlists, and
streams via the Apify YouTube Scraper actor. It replaced a .NET global tool prototype (`Youtube.Console`)
by Niels Bosma and preserves complete backward compatibility with its flags and environment variables
while upgrading the system to the SpaceCorps agentic standard.

For the manual the *agent* reads, run `youtube agent-readme` — that text lives in `src/readme.rs`
and is the tool's embedded interface for AI agents. This file is for the human or agent editing the source code.

## Commands

```bash
cargo build --release              # target/release/youtube
cargo test --locked                # unit tests + tests/cli.rs against an in-process mock server
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install on PATH
```

Use a throwaway config directory and the plaintext keystore when testing so you never touch real credentials:

```bash
export YOUTUBE_CONFIG_DIR=$(mktemp -d) YOUTUBE_SECRET_STORE=plaintext YOUTUBE_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
| --- | --- |
| `YOUTUBE_CONFIG_DIR` | Overrides the config and secrets storage directory |
| `YOUTUBE_SECRET_STORE` | Forces a keystore backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `YOUTUBE_ALLOW_PLAINTEXT_STORE=1` | Permits the unencrypted plaintext fallback when no OS keystore is available |
| `YOUTUBE_API_URL` | Overrides the Apify API base URL (`tests/cli.rs` points at its mock server) |
| `APIFY_TOKEN` | Fallback Apify API token when neither `--account` nor `--api-key` is supplied |

## Layout

```
src/
  main.rs          arg parsing, --json pre-scan, clap error formatting
  cli.rs           clap derive command hierarchy and help documentation
  commands/
    mod.rs         command dispatcher
    search.rs      youtube search command implementation
    scrape.rs      youtube scrape command implementation
    login.rs       interactive/stdin login and keystore storage
    accounts.rs    accounts add|list|test|remove management
  client.rs        blocking HTTP via ureq + rustls, status -> ErrorCode mapping
  error.rs         ErrorCode enum and Error { code, message, detail, remediation } envelope
  output.rs        YAML default (serde_norway), JSON (--json), write_error, obj! macro
  account.rs       multi-account resolution and identity probing
  config.rs        config.yaml paths, atomic writes, file permissions, cross-process locking
  secrets.rs       macOS Keychain, Linux secret-tool, Windows DPAPI, plaintext fallback
  readme.rs        agent-readme embedded operating manual and rules
tests/
  cli.rs           in-process TCP mock HTTP server integration test suite
```

## Architectural Tenets

1. **Blocking HTTP over Tokio**: A CLI typically makes 1 to a few requests. Tokio would cost more in
   binary size and cold-start latency than it could save.
2. **Native OS Keystores**: Secrets are never saved to `config.yaml` in plaintext without explicit
   user opt-in (`YOUTUBE_ALLOW_PLAINTEXT_STORE=1`).
3. **YAML Default, JSON with `--json`**: Terminal output is clear YAML by default, while `--json` produces
   clean, valid JSON suitable for `jq` and LLM tool loops.
4. **Structured Error Envelopes**: Errors on `stderr` follow a strict schema containing `error`, `code`,
   optional `detail`, and actionable `remediation` commands.
