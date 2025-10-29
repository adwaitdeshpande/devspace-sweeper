# Contributing to DevSpace Sweeper

Thanks for your interest in contributing!

## Getting started

- Requires Rust (rustup recommended). Build with:
  - `cargo build`
  - `cargo run -- scan --path .`
- Recipes live in `recipes/default.yml`.

## Development

- Run clippy and tests:
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Submit small, focused PRs.
- Add or update docs when changing behavior.

## Feature ideas

- Windows support
- Interactive TUI mode
- CI cache hints generator

## Code of Conduct

Be respectful and constructive. We welcome first-time contributors!
