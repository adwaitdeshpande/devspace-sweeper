<<<<<<< HEAD
# devspace-sweeper
=======
# DevSpace Sweeper

A tiny cross-platform CLI that finds and safely cleans dev junk across your projects to reclaim disk space and speed up tools.

## Features

- Scan for large/duplicated build outputs and caches (language-agnostic)
- Safety-first cleaning with dry-run by default
- Simple YAML recipes for patterns; easy to extend
- Markdown report generation

## Install

Build from source (requires Rust):

```bash
cargo install --path .
```

Or run in place:

```bash
cargo run -- <command> [options]
```

## Usage

```bash
# Scan current directory
devspace-sweeper scan

# Suggest cleanup candidates
devspace-sweeper suggest --path ~/dev

# Dry-run clean older than 30 days
devspace-sweeper clean --path . --max-age-days 30 --dry-run true

# Actually clean (careful!)
devspace-sweeper clean --path .

# Generate report
devspace-sweeper report --path . --output sweep-report.md
```

## Recipes

Edit `recipes/default.yml` or pass `--recipes path/to/custom.yml`.

## Roadmap

- Windows support
- .gitignore hints and CI cache suggestions
- Interactive TUI mode

## License

MIT
>>>>>>> 3679f1d (init: devspace-sweeper MVP)
