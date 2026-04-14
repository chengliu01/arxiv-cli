# Output Semantics

Read this file whenever the user asks what a command returns, what fields are included, how to parse output, or where downloaded files are saved.

## General output behavior

- Human-readable output is the default.
- `--json` switches to machine-readable JSON when supported.
- Table output is designed for terminal reading.
- JSON output is better for automation, scripting, and downstream parsing.

## Search and latest output

Default table output for `search` and `latest` focuses on short summary fields such as:

- `id`
- `title`
- `authors`
- `category`
- `published`

If `--include-abstract` is passed, abstract text is added to the returned results.

JSON behavior:

- `arxiv search --json` and `arxiv latest --json` return structured result objects
- by default, JSON omits `abstract_text`
- add `--include-abstract` if JSON should include `abstract_text`

This means an agent should not assume that `abstract_text` is always present in JSON.

## Show output

`arxiv show` is detailed output for one paper.

It includes:

- title
- authors
- abstract
- categories
- published time
- updated time
- PDF URL
- source URL
- local library state when applicable

Use `arxiv show --json` when the user needs the full metadata in a structured form.

## Download output

`arxiv download` prints success lines and saved paths.

Typical behavior:

```text
downloaded 1706.03762v7
  pdf: /.../1706.03762v7.pdf
  source: /.../arXiv-1706.03762v7
```

Important details:

- the command prints saved file paths directly
- `source:` is the extracted directory path, not the `.tar.gz` path
- if multiple IDs are downloaded, one block is printed per paper

When a user asks "where did it save the file?", the correct answer is that `download` prints the path as part of its normal output.

## Library output

`library list` returns saved entries rather than live arXiv search results.

Depending on subcommand, library output may include:

- normalized paper ID
- download flags
- PDF path
- source path
- metadata already stored in the local library

## Config and path output

- `arxiv config show` prints current config values
- `arxiv path` prints resolved config, data, library, and download paths

Use `arxiv path` when the user needs exact filesystem locations.
