# `arxiv latest`

Use `latest` when the category is already known and the user wants the newest submissions.

## Typical examples

```bash
arxiv latest cs.CL
arxiv latest cs.CL --limit 20
arxiv latest cs.CL --from 2025-01-01 --to 2025-12-31
arxiv latest cs.LG --json
arxiv latest cs.LG --include-abstract
```

## Semantics

- The category argument is required.
- Results are always sorted by submitted time descending.
- `--limit` defaults to `10`.
- `--from` and `--to` restrict the submitted-date range.
- `--include-abstract` adds abstract text to output.

## Use guidance

- Prefer this command over `search --sort submitted` when the user explicitly wants the newest papers in one category.
- This command is category-based, not institution-based.
