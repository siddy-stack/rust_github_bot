# Rust GitHub Bot

A Rust-based GitHub repository resolver that searches GitHub and intelligently determines which repository a user is referring to.

The project focuses on building a reliable repository matching system rather than simply returning the first GitHub search result.

## Features

- Direct `owner/repository` lookup
- GitHub repository search
- Query normalization
- Query expansion
- Exact repository matching
- Exact `owner/repository` matching
- Abbreviation matching
- Token-based matching
- Description matching
- Fuzzy matching
- Repository ranking
- Confidence scoring
- Ambiguity detection
- Refuses to guess when a result is unclear

## How It Works

The resolver takes a user's query, searches GitHub for possible repositories, and evaluates the candidates using multiple matching strategies.

```text
User Query
    │
    ▼
Query Normalization
    │
    ▼
Query Expansion
    │
    ▼
GitHub Search
    │
    ▼
Candidate Repositories
    │
    ▼
Matching & Scoring
    │
    ▼
Repository Ranking
    │
    ▼
Confidence Calculation
    │
    ├── Confident → Best Repository
    │
    └── Ambiguous → Show Candidates
```

The resolver does not blindly select a repository. When multiple candidates are too similar, it returns the candidates instead of making an unreliable guess.

## Examples

### Repository Name

```bash
cargo run -- vscode
```

Example result:

```text
✓ Repository found
-----------------
Name: vscode
Full name: microsoft/vscode

Match information
-----------------
Reason: Exact repository name
Score: 4000
Confidence: 98%
```

### Natural-Language Query

```bash
cargo run -- "visual studio code"
```

The resolver can recognize the abbreviation and identify:

```text
microsoft/vscode
```

### Typo

```bash
cargo run -- pyest
```

The fuzzy matching system can identify:

```text
pytest-dev/pytest
```

### Direct Repository Lookup

```bash
cargo run -- microsoft/vscode
```

This bypasses repository search and directly retrieves the specified repository.

### Ambiguous Query

```bash
cargo run -- test
```

Instead of guessing a repository, the resolver displays the strongest candidates.

## Matching System

The resolver uses several levels of matching:

| Match Type       | Description                                         |
| ---------------- | --------------------------------------------------- |
| Exact Repository | Exact `owner/repository` match                      |
| Exact Name       | Exact repository name                               |
| Abbreviation     | Recognizes common abbreviations                     |
| Prefix           | Repository name starts with the query               |
| Contains         | Repository name contains the query                  |
| Description      | Query appears in the repository description         |
| Token Match      | Individual query terms match repository information |
| Fuzzy Match      | Handles small spelling differences                  |

Results are then ranked using additional signals such as repository popularity and the difference between the strongest and second-best candidates.

## Project Structure

```text
rust_bot/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
│
├── src/
│   ├── main.rs
│   └── lib.rs
│
└── tests/
    └── resolver_tests.rs
```

### `main.rs`

Responsible for the application layer:

- Command-line arguments
- GitHub API requests
- Search results
- User-facing output

### `lib.rs`

Contains the repository resolution engine:

- Text normalization
- Query expansion
- Matching
- Scoring
- Ranking
- Fuzzy matching
- Confidence calculation
- Repository resolution

### `tests/`

Contains integration tests for the repository resolution engine.

The tests run against locally-created repository data and do not require GitHub API access.

## Testing

Run the complete test suite:

```bash
cargo test
```

Current test status:

```text
14 passed
0 failed
1 ignored
```

The ignored test represents planned future work around more advanced semantic matching.

Run ignored tests with:

```bash
cargo test -- --ignored
```

## Development

Check the project:

```bash
cargo check
```

Format the code:

```bash
cargo fmt
```

Verify formatting:

```bash
cargo fmt --check
```

Run Clippy:

```bash
cargo clippy
```

Run tests:

```bash
cargo test
```

## Requirements

- Rust
- Cargo
- Internet connection for GitHub API searches

## Current Status

**Version 0.2.0**

Version 0.2 introduces:

- Separation of application and library logic
- Improved repository resolution
- Query expansion
- Ranking and confidence scoring
- Ambiguity handling
- Integration test coverage

## License

License information will be added in a future release.
