# Rust GitHub Bot

A Rust-based GitHub repository resolver that searches GitHub and intelligently determines which repository a user is referring to.

The project focuses on building a reliable repository matching system rather than simply returning the first GitHub search result.

The application is available through both a **command-line interface** and a **web-based UI** backed by a Rust HTTP API.

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
- Prefix matching
- Contains matching
- Fuzzy matching
- Repository ranking
- Confidence scoring
- Ambiguity detection
- Ranking-based tie breaking
- Refuses to guess when a result is unclear
- REST API built with Axum
- Browser-based search UI
- Loading and error states in the UI
- Direct links to matched GitHub repositories
- API integration tests
- GitHub dependency security auditing
- Rust formatting and Clippy checks

## How It Works

The resolver takes a user's query, searches GitHub for possible repositories, and evaluates the candidates using multiple matching strategies.

```text
                         User
                          │
              ┌───────────┴───────────┐
              │                       │
              ▼                       ▼
            CLI                     Web UI
              │                       │
              │                 HTTP API / Axum
              │                       │
              └───────────┬───────────┘
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
                    ┌─────┴─────┐
                    │           │
                    ▼           ▼
                Confident    Ambiguous
                    │           │
                    ▼           ▼
             Best Repository   Reject
```

The resolver does not blindly select a repository.

When multiple candidates are too similar, it refuses to make an unreliable guess.

Ranking information can be used to resolve certain exact-name collisions when one candidate has a sufficiently stronger ranking signal.

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

The resolver expands the query and can recognize the relationship with:

```text
microsoft/vscode
```

### Typo

```bash
cargo run -- pyest
```

The resolver uses fuzzy matching to evaluate spelling differences.

A fuzzy candidate is returned only when it passes the resolver's confidence and ambiguity rules.

### Direct Repository Lookup

```bash
cargo run -- microsoft/vscode
```

This bypasses repository search and directly retrieves the specified repository.

### Ambiguous Query

```bash
cargo run -- test
```

Instead of blindly selecting a repository, the resolver can reject the result when the candidates are too similar.

## Matching System

The resolver uses several levels of matching:

| Match Type       | Description                                         |
| ---------------- | --------------------------------------------------- |
| Exact Repository | Exact `owner/repository` match                      |
| Exact Name       | Exact repository name                               |
| Abbreviation     | Recognizes supported abbreviations                  |
| Prefix           | Repository name starts with the query               |
| Contains         | Repository name contains the query                  |
| Description      | Query appears in the repository description         |
| Token Match      | Individual query terms match repository information |
| Fuzzy Match      | Handles small spelling differences                  |

Results are then ranked using additional signals such as repository popularity and the difference between the strongest and second-best candidates.

## Confidence System

The resolver assigns confidence based on the type and strength of the match.

The scoring system uses named constants for the different matching strategies and confidence levels.

The confidence calculation also considers how far the best candidate is from the next-best candidate.

This helps prevent the resolver from returning a repository when the available candidates are too similar.

## Ambiguity Handling

A core goal of the project is to avoid guessing.

For example, when several repositories receive similar matching scores:

```text
Query
  │
  ▼
Candidate A ── Score: 2500
Candidate B ── Score: 2500
  │
  ▼
Ambiguous
  │
  ▼
No confident result
```

When candidates have the same match score, ranking signals can be used to distinguish them when the ranking difference is sufficiently large.

This allows cases such as common repository names to be resolved without weakening the general ambiguity protection.

## Web API

The project exposes the repository resolver through an HTTP API built with Axum.

Start the API server:

```bash
cargo run --bin server
```

The server runs at:

```text
http://127.0.0.1:3000
```

### Search Endpoint

```text
GET /api/search?q=pytest
```

Example:

```bash
curl "http://127.0.0.1:3000/api/search?q=pytest"
```

Example response:

```json
{
  "name": "pytest",
  "full_name": "pytest-dev/pytest",
  "description": "The pytest framework makes it easy to write small tests, yet scales to support complex functional testing",
  "stars": 14521,
  "forks": 3400,
  "open_issues": 828,
  "score": 4000,
  "confidence": 98,
  "reason": "Exact repository name"
}
```

The exact GitHub repository statistics will change over time.

### API Error Handling

| Status | Meaning                                       |
| ------ | --------------------------------------------- |
| `200`  | Repository successfully resolved              |
| `400`  | Invalid or empty query                        |
| `404`  | No repository could be confidently identified |
| `500`  | GitHub API or server error                    |

## Web UI

The project includes a browser-based interface built with HTML, CSS, and JavaScript.

Start the server:

```bash
cargo run --bin server
```

Then open:

```text
http://127.0.0.1:3000/
```

The UI provides:

- Repository search
- Loading state while GitHub is queried
- Repository result cards
- Stars, forks, and open issue counts
- Match reason
- Match score
- Confidence percentage
- Direct `View on GitHub →` link
- Empty-query validation
- No-confident-match handling
- API/server error handling
- Responsive layout for smaller screens

The web UI uses the same resolver and API as the CLI rather than implementing a separate matching system.

## Project Structure

```text
rust_bot/

├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
│
├── static/
│   └── index.html
│
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── api.rs
│   │
│   └── bin/
│       └── server.rs
│
└── tests/
    ├── resolver_tests.rs
    └── api_tests.rs
```

### `main.rs`

Responsible for the command-line application:

- Command-line arguments
- Direct repository lookup
- GitHub search
- Resolver output
- Candidate display when no confident result is available

### `lib.rs`

Contains the core repository resolution engine:

- Repository model
- GitHub search helper
- Text normalization
- Query expansion
- Matching
- Scoring
- Ranking
- Fuzzy matching
- Confidence calculation
- Ambiguity handling
- Repository resolution

### `api.rs`

Contains the HTTP API layer:

- Axum router
- Application state
- Search endpoint
- API response model
- HTTP error handling
- Connection between the API and resolver

### `bin/server.rs`

Starts the Axum HTTP server and exposes the web application and API.

### `static/index.html`

Contains the browser interface:

- Search form
- Result card
- Repository statistics
- Match information
- Loading state
- Error handling
- GitHub links
- Responsive styling

### `tests/resolver_tests.rs`

Contains integration tests for the repository resolution engine.

These tests use locally-created repository data and do not require GitHub API access.

### `tests/api_tests.rs`

Contains HTTP/API integration tests covering API behavior such as:

- Successful repository searches
- Empty queries
- Unconfident results

## Testing

Run the complete test suite:

```bash
cargo test --all-targets --all-features
```

The project contains both resolver tests and API tests.

### Check Compilation

```bash
cargo check --all-targets
```

### Format Code

```bash
cargo fmt
```

### Verify Formatting

```bash
cargo fmt --all -- --check
```

### Run Clippy

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Security Audit

The project uses `cargo-audit` to check dependencies for known security vulnerabilities.

Install:

```bash
cargo install cargo-audit --locked
```

Run:

```bash
cargo audit
```

## CI

GitHub Actions are used to automatically validate the project.

The CI workflow checks:

- Rust formatting
- Compilation
- Clippy
- Tests

The security workflow checks dependencies using:

```bash
cargo audit
```

The workflows run automatically through GitHub Actions for the configured branches and pull requests.

## Requirements

- Rust
- Cargo
- Internet connection for GitHub API searches
- GitHub API access

## Running Locally

### CLI

```bash
cargo run -- pytest
```

### Web Application

```bash
cargo run --bin server
```

Then open:

```text
http://127.0.0.1:3000/
```

### API

```text
http://127.0.0.1:3000/api/search?q=pytest
```

## Current Status

**Version 0.2.0 — Current Development**

The current development work includes:

- Improved resolver scoring
- Named confidence constants
- Actual fuzzy similarity scores
- Additional ambiguity handling
- Resolver regression tests
- Reusable GitHub search logic
- Axum HTTP API
- API integration tests
- Browser-based UI
- Loading and error states
- GitHub repository links
- Improved exact-name ranking tie breaking
- Dependency updates for security advisories
- Continued CI and security auditing

## Future Work

Potential future improvements include:

- More advanced semantic matching
- Better API response models
- GitHub API rate-limit handling
- Authentication for higher GitHub API limits
- More extensive mocked API testing
- UI improvements and search history
- Additional repository metadata
- Deployment configuration

## License

License information will be added in a future release.
