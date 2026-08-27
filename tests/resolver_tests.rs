use rust_bot::{MatchReason, Repository, normalize_text, resolve_repository, score_repository};

fn repo(name: &str, full_name: &str, description: Option<&str>, stars: u32) -> Repository {
    Repository {
        name: name.to_string(),
        full_name: full_name.to_string(),
        description: description.map(str::to_string),
        stargazers_count: stars,
        forks_count: 0,
        open_issues_count: 0,
    }
}

// ============================================================
// Normalization
// ============================================================

#[test]
fn normalize_text_handles_case() {
    let result = normalize_text("PyTeSt");

    assert_eq!(result, "pytest");
}

#[test]
fn normalize_text_handles_punctuation() {
    let result = normalize_text("Visual-Studio_Code!!!");

    assert_eq!(result, "visual studio code");
}

#[test]
fn normalize_text_handles_extra_whitespace() {
    let result = normalize_text("   visual    studio     code   ");

    assert_eq!(result, "visual studio code");
}

// ============================================================
// Exact Matching
// ============================================================

#[test]
fn exact_repository_name_gets_high_score() {
    let repository = repo(
        "pytest",
        "pytest-dev/pytest",
        Some("The pytest framework"),
        14_000,
    );

    let (score, reason) = score_repository(&repository, "pytest");

    assert_eq!(score, 4000);
    assert!(matches!(reason, MatchReason::ExactName));
}

#[test]
fn exact_owner_repository_match_gets_highest_score() {
    let repository = repo(
        "pytest",
        "pytest-dev/pytest",
        Some("The pytest framework"),
        14_000,
    );

    let (score, reason) = score_repository(&repository, "pytest-dev/pytest");

    assert_eq!(score, 5000);
    assert!(matches!(reason, MatchReason::ExactRepository));
}

#[test]
fn vscode_resolves_by_exact_name() {
    let repositories = vec![
        repo(
            "vscode",
            "microsoft/vscode",
            Some("Visual Studio Code"),
            189_605,
        ),
        repo(
            "vscode-icons",
            "vscode-icons/vscode-icons",
            Some("Icons for Visual Studio Code"),
            5_000,
        ),
    ];

    let result = resolve_repository(&repositories, "vscode");

    assert!(result.is_some());

    let result = result.unwrap();

    assert_eq!(result.repository.full_name, "microsoft/vscode");

    assert!(matches!(result.reason, MatchReason::ExactName));

    assert_eq!(result.score, 4000);
}

// ============================================================
// Abbreviation / Semantic Matching
// ============================================================

#[test]
fn visual_studio_code_resolves_to_vscode() {
    let repositories = vec![
        repo(
            "vscode",
            "microsoft/vscode",
            Some("Visual Studio Code"),
            189_605,
        ),
        repo(
            "visual-studio-code",
            "dracula/visual-studio-code",
            Some("Dark theme for Visual Studio Code"),
            878,
        ),
    ];

    let result = resolve_repository(&repositories, "visual studio code");

    assert!(result.is_some());

    let result = result.unwrap();

    assert_eq!(result.repository.full_name, "microsoft/vscode");

    assert!(matches!(result.reason, MatchReason::Abbreviation));

    assert_eq!(result.score, 3500);
}

#[test]
fn description_phrase_is_detected() {
    let repository = repo(
        "some-project",
        "example/some-project",
        Some("A powerful visual studio code extension"),
        100,
    );

    let (score, reason) = score_repository(&repository, "visual studio code");

    assert_eq!(score, 1200);

    assert!(matches!(reason, MatchReason::DescriptionPhrase));
}

// ============================================================
// Fuzzy Matching
// ============================================================

#[test]
fn typo_pyest_resolves_to_pytest() {
    let repositories = vec![
        repo(
            "pytest",
            "pytest-dev/pytest",
            Some("The pytest framework"),
            14_000,
        ),
        repo(
            "pytest-cov",
            "pytest-dev/pytest-cov",
            Some("Coverage plugin for pytest"),
            2_000,
        ),
    ];

    let result = resolve_repository(&repositories, "pyest");

    assert!(result.is_some());

    let result = result.unwrap();

    assert_eq!(result.repository.full_name, "pytest-dev/pytest");

    assert!(matches!(result.reason, MatchReason::Fuzzy));
}

// ============================================================
// Ambiguity
// ============================================================

#[test]
fn ambiguous_query_does_not_guess() {
    let repositories = vec![
        repo(
            "test-one",
            "example/test-one",
            Some("A test repository"),
            100,
        ),
        repo(
            "test-two",
            "example/test-two",
            Some("Another test repository"),
            100,
        ),
        repo(
            "test-three",
            "example/test-three",
            Some("Yet another test repository"),
            100,
        ),
    ];

    let result = resolve_repository(&repositories, "test");

    assert!(
        result.is_none(),
        "Resolver should refuse to guess when candidates are equally strong"
    );
}

// ============================================================
// Confidence
// ============================================================

#[test]
fn exact_match_has_high_confidence() {
    let repositories = vec![
        repo(
            "pytest",
            "pytest-dev/pytest",
            Some("The pytest framework"),
            14_000,
        ),
        repo(
            "pytest-cov",
            "pytest-dev/pytest-cov",
            Some("Coverage plugin for pytest"),
            2_000,
        ),
    ];

    let result = resolve_repository(&repositories, "pytest");

    assert!(result.is_some());

    let result = result.unwrap();

    assert!(
        result.confidence >= 95,
        "Expected high confidence, got {}%",
        result.confidence
    );
}

#[test]
fn abbreviation_match_has_high_confidence() {
    let repositories = vec![
        repo(
            "vscode",
            "microsoft/vscode",
            Some("Visual Studio Code"),
            189_605,
        ),
        repo(
            "visual-studio-code",
            "dracula/visual-studio-code",
            Some("Dark theme for Visual Studio Code"),
            878,
        ),
    ];

    let result = resolve_repository(&repositories, "visual studio code");

    assert!(result.is_some());

    let result = result.unwrap();

    assert!(
        result.confidence >= 90,
        "Expected high confidence, got {}%",
        result.confidence
    );
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn repository_without_description_does_not_panic() {
    let repository = repo("some-project", "example/some-project", None, 100);

    let result = score_repository(&repository, "something");

    assert_eq!(result.0, 0);
}

#[test]
fn empty_query_does_not_resolve() {
    let repositories = vec![repo(
        "pytest",
        "pytest-dev/pytest",
        Some("The pytest framework"),
        14_000,
    )];

    let result = resolve_repository(&repositories, "");

    assert!(result.is_none());
}

// ============================================================
// Known Limitations / Future Work
// ============================================================

#[test]
#[ignore = "Future: improve semantic matching for arbitrary product aliases"]
fn arbitrary_product_alias_should_resolve() {
    let repositories = vec![repo(
        "vscode",
        "microsoft/vscode",
        Some("Visual Studio Code"),
        189_605,
    )];

    let result = resolve_repository(&repositories, "code editor");

    assert!(result.is_some());
}
