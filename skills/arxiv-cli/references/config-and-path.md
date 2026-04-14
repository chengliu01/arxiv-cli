# `arxiv config` and `arxiv path`

Use these commands to inspect or configure where the CLI stores data and downloads.

## Typical examples

```bash
arxiv config show
arxiv config set-download-dir ~/Documents/papers/arxiv
arxiv path
```

## Semantics

- `config show` prints current configuration values
- `config set-download-dir <DIR>` registers a default download folder for future downloads
- `path` prints resolved config, data, library, and download paths

## Use guidance

- If the user wants one-off output redirection, use `download --output`.
- If the user wants a persistent default location, use `config set-download-dir`.
