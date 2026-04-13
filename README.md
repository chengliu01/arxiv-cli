# arxiv-cli

A Rust CLI for searching arXiv, listing the latest papers in a category, downloading papers, and managing a local paper library.

## Install

This project is not published to crates.io yet. Install it from source:

```bash
git clone <your-repo-url> arxiv-cli
cd arxiv-cli
cargo install --path .
```

Verify the install:

```bash
arxiv --help
```

Run without global install during development:

```bash
cargo run -- --help
```

## What It Can Do

- Search arXiv by keyword, title, author, category, and submitted date range
- List the newest papers in a category, always sorted by submission time
- Show full paper metadata including abstract, PDF URL, and source URL
- Download PDF, LaTeX source, or both
- Automatically extract downloaded source archives by default
- Register a default download directory
- Download multiple papers in parallel
- Maintain a local JSON-backed library
- Output either human-readable tables or JSON

## Quick Start

```bash
# keyword search
arxiv search "diffusion models"

# title-only search
arxiv search --title "skill"

# search with filters
arxiv search "transformer" --author "Vaswani" --category cs.CL --from 2024-01-01 --to 2024-12-31

# list newest papers in a category
arxiv latest cs.CL --limit 10

# show one paper
arxiv show 1706.03762

# download PDF and source
arxiv download 1706.03762 2401.12345 --format both --jobs 4

# set a default download directory
arxiv config set-download-dir ~/Documents/papers/arxiv
```

## Command Guide

### `arxiv search`

Use `search` when you want flexible retrieval across general arXiv text fields.

Examples:

```bash
arxiv search "skill"
arxiv search "diffusion models" --limit 20
arxiv search "llm" --category cs.CL --sort submitted
arxiv search --title "large language model"
arxiv search --author "Yann LeCun"
arxiv search "transformer" --from 2025-01-01 --to 2025-12-31
arxiv search "skill" --include-abstract
arxiv search "skill" --json
arxiv search "skill" --json --include-abstract
```

Key parameters:

- `QUERY`: optional keyword query. Multiple words are combined with `AND`
- `--title <TITLE>`: search title only
- `--author <AUTHOR>`: filter by author name
- `--category <CATEGORY>`: filter by arXiv category such as `cs.CL`, `cs.LG`, `math.PR`
- `--from <YYYY-MM-DD>` / `--to <YYYY-MM-DD>`: submitted-date range
- `--sort relevance|updated|submitted`: choose ranking field
- `--order asc|desc`: choose sort direction
- `--limit <N>`: number of results to return
- `--start <N>`: pagination offset
- `--include-abstract`: include abstracts in table output and JSON output
- `--json`: print machine-readable JSON. By default JSON omits `abstract_text`; add `--include-abstract` to include it

Search behavior:

- `arxiv search "diffusion models"` means `diffusion AND models`
- `arxiv search --title "diffusion models"` searches that phrase in the title field
- Plain keyword search uses arXiv's broad text search fields, which include title/abstract-style matching

### `arxiv latest`

Use `latest` when you already know the category and only want the newest submissions.

Examples:

```bash
arxiv latest cs.CL
arxiv latest cs.CL --limit 20
arxiv latest cs.CL --from 2025-01-01 --to 2025-12-31
arxiv latest math.PR --include-abstract
arxiv latest cs.LG --json
arxiv latest cs.LG --json --include-abstract
```

Key parameters:

- `<CATEGORY>`: required arXiv category, such as `cs.CL`, `cs.LG`, `stat.ML`
- `--limit <N>`: number of papers to return, default `10`
- `--from <YYYY-MM-DD>` / `--to <YYYY-MM-DD>`: submitted-date range
- `--include-abstract`: include abstracts in table output and JSON output
- `--json`: print JSON instead of a table. By default JSON omits `abstract_text`; add `--include-abstract` to include it

Behavior:

- `latest` is always sorted by submitted time descending
- It is designed for category feeds, not institution/affiliation search

### `arxiv show`

Use `show` to inspect one paper in detail.

```bash
arxiv show 1706.03762
arxiv show 1706.03762 --json
```

Returned information includes:

- title
- authors
- abstract
- categories
- published / updated time
- PDF URL
- source URL
- local library status if the paper is already saved

### `arxiv download`

Use `download` to save PDF, source, or both to disk.

Examples:

```bash
arxiv download 1706.03762
arxiv download 1706.03762 --format source
arxiv download 1706.03762 2401.12345 --format both --jobs 4
arxiv download 1706.03762 --output ~/Downloads/arxiv
arxiv download 1706.03762 --force
arxiv download 1706.03762 --no-library-update
```

Key parameters:

- `[IDS]...`: one or more arXiv IDs
- `--format pdf|source|both`: what to download
- `--output <DIR>`: override the default download directory for this command
- `--jobs <N>`: parallel download concurrency, default `4`
- `--force`: re-download even if the file already exists
- `--no-library-update`: skip writing download status back to the local library

Output behavior:

- successful downloads print the paper ID
- the CLI also prints the saved file path for PDF and/or source
- source downloads are extracted by default; the printed `source:` path is the extracted directory

Example output:

```text
downloaded 1706.03762v7
  pdf: /.../1706.03762v7.pdf
  source: /.../arXiv-1706.03762v7 # tar.gz is extracted to this directory automatically
```

### `arxiv library`

Use `library` to maintain a local index of papers you care about.

Examples:

```bash
arxiv library add 1706.03762
arxiv library list
arxiv library list --downloaded-only
arxiv library list --category cs.CL
arxiv library list --author "Vaswani"
arxiv library show 1706.03762
arxiv library remove 1706.03762
arxiv library remove 1706.03762 --purge-files
```

Subcommands:

- `library add <ID...>`: save metadata into the local library without downloading
- `library list`: list library entries
- `library show <ID>`: show one local entry with file paths and status
- `library remove <ID>`: remove one entry from the index
- `--purge-files`: remove associated local files too

### `arxiv config` and `arxiv path`

Use `config` to inspect or update default behavior.

Examples:

```bash
arxiv config show
arxiv config set-download-dir ~/Documents/papers/arxiv
arxiv path
```

Use cases:

- register a default download folder once, then omit `--output`
- print resolved config/data/library/download paths for scripting

## Common Workflows

### 1. Search then inspect

```bash
arxiv search "test-time scaling" --category cs.CL --limit 5
arxiv show 2501.12345
```

### 2. Follow the latest papers in one category

```bash
arxiv latest cs.CL --limit 20
arxiv latest cs.CL --from 2026-01-01 --to 2026-04-01
```

### 3. Download a paper set for local reading

```bash
arxiv config set-download-dir ~/Documents/papers/arxiv
arxiv download 1706.03762 2401.12345 --format both --jobs 4
```

### 4. Build a small local paper library

```bash
arxiv library add 1706.03762 2401.12345
arxiv library list
arxiv library show 1706.03762
```

## Storage

By default the CLI stores:

- `config.toml` in the platform config directory
- `library.json` in the platform data directory
- downloaded files under `papers/<arxiv_id>/`

Print the resolved paths with:

```bash
arxiv path
```

## Notes

- `latest` currently works on arXiv categories, not institution/affiliation feeds
- Search date filters apply to submitted time
- If you want script-friendly output, prefer `--json`

## Test

```bash
cargo test
```
