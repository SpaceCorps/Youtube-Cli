---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the YouTube CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for YouTube CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the YouTube CLI.

## Overview
The YouTube CLI interfaces with the Apify REST API (v2) to drive the YouTube scraper actor. Authentication is token-based using API tokens issued through your Apify account. Tokens can be stored in the host operating system's native keychain, supplied directly via environment variables, or passed on standard input.

## Prerequisites
- An Apify account ([apify.com](https://apify.com))
- A valid API token generated from your Apify integrations page (`https://console.apify.com/account/integrations`)
- YouTube CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/Youtube-Cli --locked`)

## Authentication Flow

### Interactive Browser Login (`youtube login`)
The recommended flow for local developer machines:
```bash
youtube login [account_name]
```
1. The CLI launches your system browser to `https://console.apify.com/account/integrations`.
2. You copy your Apify personal API token.
3. Paste the token into the CLI prompt (input characters are masked).
4. The CLI validates the key with a live request to `GET /v2/users/me`.
5. Upon confirmation, the key is securely saved to the native OS keyring under the account name (defaults to `default`).

### Non-Interactive / Headless Login
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
echo "$APIFY_TOKEN" | youtube login [account_name] --api-key-stdin
```
Or pass the token directly as a CLI flag:
```bash
youtube login [account_name] --api-key "$APIFY_TOKEN"
```

## Environment Variables
The CLI checks the environment for credentials when no keychain account is specified:
- `APIFY_TOKEN`: Fallback API token used if no `--account` or `--api-key` is explicitly selected.
- `YOUTUBE_API_KEY`: Alias fallback variable for convenience.

## Multi-Account Management
Switch or verify accounts using:
```bash
youtube accounts list --check
youtube accounts test [account_name]
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized JSON error payloads:
- `auth_required`: No token provided or token expired.
- `no_account`: Specified account does not exist in keystore.
- `invalid_input`: Key format rejected by validation check.
- `rate_limited`: Apify API rate limits reached.

## Security Best Practices
1. **Never Commit Tokens**: Keep `.env` or plaintext token files out of version control.
2. **Use OS Keystore**: The CLI automatically utilizes macOS Keychain, Windows DPAPI, or Linux Secret Service.
3. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture error codes.
