---
name: arxiv-cli
description: Use when an agent needs to operate the arxiv CLI as a user tool: searching papers, listing latest category papers, showing metadata, downloading PDF or source, managing the local library, or configuring default paths. This skill is organized as an overview plus per-command reference files so the agent can load only the relevant command docs and output semantics.
---

# arxiv CLI

Use this skill when the task is about using the installed `arxiv` command, choosing the right subcommand, explaining flags, or interpreting command output.

## arxiv-cli installation

```bash
npm install -g arxiv-cli
```

## How to use this skill

Read this file first, then load only the relevant reference files from `references/`.

Always read:

- `references/output.md` when the task depends on what the command prints, returns in JSON, or saves to disk

Read one or more of these based on the task:

- `references/search.md` for `arxiv search`
- `references/latest.md` for `arxiv latest`
- `references/show.md` for `arxiv show`
- `references/download.md` for `arxiv download`
- `references/library.md` for `arxiv library`
- `references/config-and-path.md` for `arxiv config` and `arxiv path`
- `references/workflows.md` for common multi-step usage patterns

## Global rules

- The executable name is `arxiv`.
- Date filters use `YYYY-MM-DD`.
- Prefer exact shell commands in your answers.
- If the user wants machine-readable output, prefer `--json`.
- If the user wants search summaries or abstracts in result lists, include `--include-abstract`.
- If the user wants newest papers in a category, prefer `arxiv latest` over `arxiv search --sort submitted`.

## Minimal routing guide

- Discover papers broadly: `arxiv search`
- Get newest papers in one category: `arxiv latest`
- Inspect one paper deeply: `arxiv show`
- Save PDF or source locally: `arxiv download`
- Work with saved metadata or downloaded file records: `arxiv library`
- Set or inspect local storage defaults: `arxiv config` and `arxiv path`
