# wger-cli

A strict Rust CLI and MCP stdio server for the [wger](https://wger.de) REST API v2. Lists, creates, updates, and deletes resources; reads routine structures, workout logs, and translations. Designed for terminal-first use and agent integration over JSON-RPC stdio.

[![CI](https://github.com/unchain0/wger-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/unchain0/wger-cli/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange.svg)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Real output

```text
$ cargo run -- resources
[
  "routine", "day", "slot", "slot-entry",
  "workoutlog", "workoutsession",
  ...
]
```
See `docs/assets/resources-output.txt` for the full captured output.

## Features
- CLI (`--json`) + MCP stdio server (`wger mcp`)
- List, get, create, update, delete across all wger resources
- Routine sub-views and workout-log filters
- Translation creation/update for exercises
- Strict lint (clippy pedantic, warnings as errors)

## Quick start
```bash
git clone https://github.com/unchain0/wger-cli.git
cd wger-cli
cargo build --release
export WGER_API_TOKEN="<token>"
./target/release/wger resources
./target/release/wger list exercise --all
```

## Usage / examples
See README commands above and `docs/assets/`. Env vars: `WGER_API_TOKEN` (required for writes), `WGER_BASE_URL` (default `https://wger.de`). See `.env.example`.

## Architecture
Rust binary; `ureq` for HTTP; `serde_json` for data; `clap` for CLI; custom newline-delimited JSON-RPC over stdio for MCP.

## Development
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings --enable pedantic`
- `cargo test`
- Coverage threshold enforced in CI; `cargo audit` for dependency audit; `gitleaks` for secret scanning.

## CI / quality
Pinned action SHAs; least-priv `permissions: contents: read`; concurrency with cancel-in-progress; required lint, type-check, test, audit, and secret-scan jobs.

## Roadmap
Keep clippy strict; add UUIDv7 for any generated IDs; CSPRNG for any randomness.

## License
MIT
