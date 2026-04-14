# `arxiv search`

Use `search` for flexible discovery across arXiv text fields and filters.

## Typical examples

```bash
arxiv search "diffusion models"
arxiv search --title "large language model"
arxiv search --author "Yann LeCun"
arxiv search "transformer" --category cs.CL --from 2025-01-01 --to 2025-12-31
arxiv search "skill" --sort submitted --limit 20
arxiv search "skill" --json --include-abstract
```

## Semantics

- Plain keyword search uses broad arXiv text matching.
- Multiple words in `QUERY` are combined with `AND`.
- `--title` searches the title field only.
- `--author` filters by author name.
- `--category` filters by arXiv category such as `cs.CL`, `cs.LG`, `math.PR`.
- `--from` and `--to` filter by submitted date range.
- `--sort` supports `relevance`, `updated`, or `submitted`.
- `--order` supports `asc` or `desc`.
- `--limit` controls result count.
- `--start` controls pagination offset.
- `--include-abstract` adds abstract text to result output.

## Use guidance

- If the user wants the newest papers in a known category, prefer `latest` instead.
- If the user wants only title matching, prefer `--title`.
- If the user wants structured output for scripting, add `--json`.
