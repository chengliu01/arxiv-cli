# `arxiv library`

Use `library` to work with locally saved metadata and file records.

## Typical examples

```bash
arxiv library add 1706.03762
arxiv library list
arxiv library list --downloaded-only
arxiv library list --category cs.CL
arxiv library show 1706.03762
arxiv library remove 1706.03762
arxiv library remove 1706.03762 --purge-files
```

## Subcommands

- `library add <ID...>` saves metadata without downloading files
- `library list` lists local entries
- `library show <ID>` shows one local entry
- `library remove <ID>` removes one local entry from the index

## Filters and options

- `--downloaded-only` limits results to entries with downloaded files
- `--category` filters local entries by category
- `--author` filters local entries by author
- `--purge-files` also deletes associated local files when removing an entry
