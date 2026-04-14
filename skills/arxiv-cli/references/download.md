# `arxiv download`

Use `download` to save PDF, source, or both to disk.

## Typical examples

```bash
arxiv download 1706.03762
arxiv download 1706.03762 --format source
arxiv download 1706.03762 2401.12345 --format both --jobs 4
arxiv download 1706.03762 --output ~/Downloads/arxiv
arxiv download 1706.03762 --force
arxiv download 1706.03762 --no-library-update
```

## Semantics

- Accepts one or more arXiv IDs.
- `--format` supports `pdf`, `source`, or `both`.
- `--jobs` controls parallel download concurrency.
- `--output` overrides the default download directory for that command only.
- `--force` re-downloads even if files already exist.
- `--no-library-update` skips writing download status into the local library.

## Output details

- On success, the command prints saved file paths.
- Source downloads are extracted by default.
- The printed `source:` path is the extracted directory.

## Use guidance

- If the user asks for a default download directory, recommend `arxiv config set-download-dir`.
- If the user asks where the file was stored, reference the printed path from command output.
