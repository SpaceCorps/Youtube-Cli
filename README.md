# Youtube.Console

CLI for scraping YouTube videos, channels, playlists, and search results via Apify. YAML-first output optimized for LLM agent consumption.

## Installation

```bash
dotnet tool install -g Youtube.Console
```

## Prerequisites

Set your Apify API token:

```bash
export APIFY_TOKEN=your-token-here
```

Or pass it per-command with `--api-key`.

## Usage

### Search

```bash
# Search for videos
youtube search "dotnet 10 new features"

# Limit results
youtube search "typescript tutorial" --max-results 5

# Include Shorts and streams
youtube search "cooking" --max-results 5 --max-shorts 3 --max-streams 2

# Sort and filter
youtube search "rust programming" --sort date --date month --length between420

# HD only
youtube search "4k nature" --hd --4k

# With subtitles download
youtube search "conference talk" --max-results 3 --download-subs --subs-lang en

# Only recent videos
youtube search "breaking news" --since "1 day"
```

### Scrape

```bash
# Scrape a single video
youtube scrape "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Scrape a channel (latest 10 videos)
youtube scrape "https://www.youtube.com/@Fireship" --max-results 10

# Scrape a playlist
youtube scrape "https://www.youtube.com/playlist?list=PLObrtcm1Kw6PmbXg8bmfJN-o2Hgx8sidf"

# Channel sorted by popularity
youtube scrape "https://www.youtube.com/@Fireship" --sort POPULAR --max-results 5

# With subtitles
youtube scrape "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --download-subs

# Only videos from last week
youtube scrape "https://www.youtube.com/@Fireship" --since "7 days"
```

### Common options

```bash
--api-key <KEY>        Apify token (or set APIFY_TOKEN env var)
--max-results <N>      Maximum regular videos (default: 10)
--max-shorts <N>       Maximum Shorts (default: 0)
--max-streams <N>      Maximum streams (default: 0)
--download-subs        Download subtitles
--subs-lang <LANG>     Subtitle language (default: en)
--subs-format <FMT>    Subtitle format: srt, vtt, xml, plaintext (default: srt)
--auto-subs            Prefer auto-generated subtitles
--since <DATE>         Only videos after this date (e.g. 2025-01-01 or '7 days')
```

## License

MIT
