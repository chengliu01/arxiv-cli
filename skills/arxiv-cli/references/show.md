# `arxiv show`

Use `show` to inspect one paper in full detail.

## Typical examples

```bash
arxiv show 1706.03762
arxiv show 1706.03762 --json
```

## Use guidance

- Use this after `search` or `latest` when the user wants full metadata for a single paper.
- Prefer `--json` when the output will be parsed or reused programmatically.

## Included information

- title
- authors
- abstract
- categories
- published and updated timestamps
- PDF URL
- source URL
- local library status if the paper is already saved
