---
title: "YouTube CLI"
description: "A blazing fast native command-line tool and agent interface for searching and scraping YouTube videos, channels, playlists, and subtitles via Apify. Built in Rust for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Youtube-Cli/index.md"
---

# YouTube CLI

A blazing fast native command-line tool and agent interface for searching and scraping YouTube videos, channels, playlists, and subtitles via Apify. Built in Rust 2024 for developers and autonomous AI agents.

## Quickstart

```bash
# Authenticate interactively via browser token flow
youtube login

# Or non-interactively in headless CI/CD environments
echo "$APIFY_TOKEN" | youtube login --api-key-stdin

# Search YouTube
youtube search "rust programming tutorial" --max-results 5

# Scrape a video with subtitles
youtube scrape "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --download-subs
```

## Features

- **Blazing Fast Native Rust**: Sub-3ms cold start times with zero runtime dependencies.
- **AI Agent Native**: Structured YAML output by default, raw JSON via `--json`, and standardized error envelopes.
- **Secure Keystore Integration**: Token storage in native macOS Keychain, Windows DPAPI, and Linux Secret Service.
- **Deep Scraping & Search**: Videos, Shorts, Livestreams, Channel catalogs, Playlists, and caption downloads.

## When to Use This CLI

Use the `youtube` CLI whenever you need to:
- Search YouTube for videos, shorts, and live streams with exact sorting and date filters.
- Scrape channel catalogs and playlists to extract view counts, durations, and metadata.
- Download subtitles (SRT, VTT, XML, or plaintext) for NLP, RAG, and AI agent consumption.
- Automate video workflows in LLM tool loops with deterministic exit codes.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Youtube-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Youtube-Cli/llms-full.txt)
- [Pricing](https://spacecorps.github.io/Youtube-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/Youtube-Cli/auth.md)
- [GitHub Repository](https://github.com/SpaceCorps/Youtube-Cli)
