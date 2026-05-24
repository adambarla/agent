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

## Checks

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
