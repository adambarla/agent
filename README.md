# Agent Harness Experiments

Small Rust CLI agent using an OpenAI-compatible chat completions API.

## Setup

Copy `.env.example` to `.env` and fill in:

```sh
API_KEY=...
CHAT_COMPLETIONS_URL=https://api.deepseek.com/chat/completions
```

Run:

```sh
cargo run
```

## Architecture

See [docs/architecture.md](docs/architecture.md) for the planned harness design.

## Checks

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
