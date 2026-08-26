# Rust GitHub Bot

A Rust-based GitHub repository resolver that searches GitHub and attempts to identify the repository a user is referring to.

## Features

- Direct `owner/repository` lookup
- GitHub repository search
- Query normalization
- Query expansion
- Exact repository-name matching
- Abbreviation matching
- Token matching
- Fuzzy matching
- Repository ranking
- Confidence scoring
- Ambiguity detection
- Refuses to guess when the result is unclear

## Examples

Search by repository name:

```bash
cargo run -- vscode
```

Search with a natural-language name

```bash
cargo run -- "visual studio code"
```

Search for a specific project like pytest

```bash
cargo run -- pytest
```

Search for Direct Repository Lookup

```bash
cargo run -- microsoft/vscode
```

## Current Status

Version 1.0 : repository search and resolution engine implemented
