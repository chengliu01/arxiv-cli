# Common Workflows

Use this file when the user is asking for a multi-step flow instead of a single command.

## Search, inspect, then download

```bash
arxiv search "test-time scaling" --category cs.CL --limit 5
arxiv show 2501.12345
arxiv download 2501.12345 --format both
```

## Track a category feed

```bash
arxiv latest cs.CL --limit 20
arxiv latest cs.CL --from 2025-01-01 --to 2025-01-31 --json
```

## Build a local library without downloading immediately

```bash
arxiv library add 1706.03762 2401.12345
arxiv library list
```

## Download several papers in parallel

```bash
arxiv download 1706.03762 2401.12345 2501.12345 --format both --jobs 4
```
