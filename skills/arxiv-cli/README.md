# arxiv-cli

`arxiv-cli` is a usage-focused skill for agents that need to operate the `arxiv` command-line tool to search papers, list the latest papers in a category, inspect metadata, download PDFs or source archives, and manage a local paper library.

It is designed for cases where the agent should choose the right `arxiv` command and flags quickly, explain command behavior clearly, and handle output details such as JSON fields, abstract inclusion, saved download paths, and local library semantics.

This skill is organized as a lightweight entry point plus per-command reference files so the agent can load only the relevant documentation for `search`, `latest`, `show`, `download`, `library`, `config`, `path`, and output interpretation.
