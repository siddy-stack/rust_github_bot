use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    description: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    open_issues_count: u32,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<Repository>,
}

#[derive(Debug, Clone, Copy)]
enum MatchReason {
    ExactRepository,
    ExactName,
    Prefix,
    Contains,
    DescriptionPhrase,
    TokenMatch,
    Abbreviation,
    Fuzzy,
}

impl MatchReason {
    fn description(&self) -> &'static str {
        match self {
            MatchReason::ExactRepository => "Exact owner/repository match",
            MatchReason::ExactName => "Exact repository name",
            MatchReason::Prefix => "Repository name starts with query",
            MatchReason::Contains => "Repository name contains query",
            MatchReason::DescriptionPhrase => "Exact phrase found in description",
            MatchReason::TokenMatch => "Strong token match",
            MatchReason::Abbreviation => "Abbreviation match",
            MatchReason::Fuzzy => "Fuzzy name match",
        }
    }
}

struct MatchResult<'a> {
    repository: &'a Repository,
    score: u32,
    confidence: u32,
    reason: MatchReason,
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];

    for (i, a_char) in a.iter().enumerate() {
        current[0] = i + 1;

        for (j, b_char) in b.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };

            current[j + 1] = (current[j] + 1)
                .min(previous[j + 1] + 1)
                .min(previous[j] + cost);
        }

        std::mem::swap(&mut previous, &mut current);
    }

    previous[b.len()]
}

fn similarity(a: &str, b: &str) -> u32 {
    if a.is_empty() || b.is_empty() {
        return 0;
    }

    let distance = levenshtein(a, b);
    let max_length = a.chars().count().max(b.chars().count());

    if distance > max_length {
        return 0;
    }

    (((max_length - distance) as f64 / max_length as f64) * 1000.0) as u32
}

fn abbreviation_matches(query: &str, text: &str) -> bool {
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let text_words: Vec<&str> = text.split_whitespace().collect();

    if query_words.len() < 2 || text_words.len() < query_words.len() {
        return false;
    }

    for start in 0..=text_words.len() - query_words.len() {
        let mut matched = true;

        for (query_word, text_word) in query_words.iter().zip(&text_words[start..]) {
            if query_word.len() <= 3 {
                let first_letter = text_word.chars().next();

                if first_letter != query_word.chars().next() {
                    matched = false;
                    break;
                }
            } else if !text_word.starts_with(query_word) {
                matched = false;
                break;
            }
        }

        if matched {
            return true;
        }
    }

    false
}

fn abbreviation_name_matches(query: &str, name: &str) -> bool {
    let query = normalize_text(query);
    let name = normalize_text(name);

    match query.as_str() {
        "vs code" | "visual studio code" => name == "vscode",
        _ => false,
    }
}

fn score_repository(repo: &Repository, query: &str) -> (u32, MatchReason) {
    let query = normalize_text(query);
    let name = normalize_text(&repo.name);
    let full_name = normalize_text(&repo.full_name);

    let description = normalize_text(repo.description.as_deref().unwrap_or(""));

    // 1. Exact owner/repository match
    if full_name == query {
        return (5000, MatchReason::ExactRepository);
    }

    // 2. Exact repository name
    if name == query {
        // For a simple one-word query, an exact repository name
        // is extremely strong evidence.
        if query.split_whitespace().count() == 1 {
            return (4000, MatchReason::ExactName);
        }

        // For multi-word natural-language queries, don't automatically
        // treat the repository name as the definitive answer.
        return (3000, MatchReason::ExactName);
    }
    // Abbrevation Name Matches (e.g.. "visual studio code" -> "vscode")
    if abbreviation_name_matches(&query, &name) {
        return (3500, MatchReason::Abbreviation);
    }
    // 3. Prefix
    if name.starts_with(&query) {
        return (2500, MatchReason::Prefix);
    }

    // 4. Name contains query
    if name.contains(&query) {
        return (1500, MatchReason::Contains);
    }

    // 5. Exact phrase in description
    if description.contains(&query) {
        return (1200, MatchReason::DescriptionPhrase);
    }

    // 6. Abbreviation matching
    if abbreviation_matches(&query, &description) {
        return (1100, MatchReason::Abbreviation);
    }

    // 7. Token matching
    let query_words: Vec<&str> = query.split_whitespace().collect();

    if !query_words.is_empty() {
        let matched_words = query_words
            .iter()
            .filter(|word| {
                name.split_whitespace().any(|name_word| {
                    name_word.contains(**word) || similarity(name_word, word) >= 750
                }) || description.split_whitespace().any(|description_word| {
                    description_word.contains(**word) || similarity(description_word, word) >= 750
                })
            })
            .count();

        if matched_words > 0 {
            let percentage = (matched_words * 100) / query_words.len();

            if percentage == 100 {
                return (1000, MatchReason::TokenMatch);
            }

            if percentage >= 50 {
                return (700, MatchReason::TokenMatch);
            }
        }
    }

    // 8. Fuzzy repository-name matching
    let fuzzy_score = similarity(&name, &query);

    if fuzzy_score >= 800 {
        return (600, MatchReason::Fuzzy);
    }

    (0, MatchReason::Fuzzy)
}

fn ranking_score(repo: &Repository, query: &str) -> u64 {
    let query = normalize_text(query);
    let name = normalize_text(&repo.name);
    let description = normalize_text(repo.description.as_deref().unwrap_or(""));

    let mut score = repo.stargazers_count as u64;

    if name == query {
        score += 100_000;
    } else if name.contains(&query) {
        score += 50_000;
    }
    if abbreviation_name_matches(&query, &name) {
        score += 75_000;
    }
    if description.contains(&query) {
        score += 10_000;
    }

    if abbreviation_matches(&query, &description) {
        score += 25_000;
    }

    score
}

fn calculate_confidence(best_score: u32, second_score: u32) -> u32 {
    if best_score == 0 {
        return 0;
    }

    let gap_ratio = if second_score == 0 {
        1.0
    } else {
        (best_score.saturating_sub(second_score) as f64) / best_score as f64
    };

    let base = match best_score {
        5000.. => 99,
        4000..=4999 => 97,
        3500..=3999 => 92,
        3000..=3499 => 87,
        2500..=2999 => 78,
        1500..=2499 => 65,
        1000..=1499 => 55,
        700..=999 => 45,
        600..=699 => 35,
        _ => 20,
    };

    let uniqueness_bonus = (gap_ratio * 5.0) as u32;

    (base + uniqueness_bonus).min(100)
}

fn resolve_repository<'a>(repositories: &'a [Repository], query: &str) -> Option<MatchResult<'a>> {
    let mut scored: Vec<_> = repositories
        .iter()
        .map(|repo| {
            let (score, reason) = score_repository(repo, query);
            let ranking = ranking_score(repo, query);

            (repo, score, reason, ranking)
        })
        .filter(|(_, score, _, _)| *score > 0)
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.3.cmp(&a.3)));

    if scored.is_empty() {
        return None;
    }

    let best = &scored[0];

    let second_score = scored.get(1).map(|result| result.1).unwrap_or(0);

    let confidence = calculate_confidence(best.1, second_score);

    // Exact matches are always trusted.
    if best.1 >= 4000 && best.1 > second_score {
        return Some(MatchResult {
            repository: best.0,
            score: best.1,
            confidence,
            reason: best.2,
        });
    }

    // Strong semantic match.
    //
    // If several repositories have the same primary score,
    // ranking_score() determines which is the best candidate.
    if best.1 >= 1000 {
        let ranking_gap = best
            .3
            .saturating_sub(scored.get(1).map(|result| result.3).unwrap_or(0));

        if best.1 > second_score || ranking_gap >= 50_000 {
            return Some(MatchResult {
                repository: best.0,
                score: best.1,
                confidence,
                reason: best.2,
            });
        }
    }

    None
}

fn expand_query(query: &str) -> Vec<String> {
    let normalized = normalize_text(query);

    let mut queries = vec![normalized.clone()];

    // Common abbreviation: "VS Code" -> "vscode"
    if normalized == "vs code" {
        queries.push("vscode".to_string());
    }

    // Common abbreviation: "Visual Studio Code" -> "vscode"
    if normalized == "visual studio code" {
        queries.push("vscode".to_string());
    }

    // Search the original phrase as an exact phrase too.
    if normalized.split_whitespace().count() > 1 {
        queries.push(format!("\"{}\"", normalized));
    }

    queries
}

async fn search_repositories(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
    let queries = expand_query(query);

    let mut repositories: HashMap<String, Repository> = HashMap::new();

    for search_query in queries {
        let response: SearchResponse = client
            .get("https://api.github.com/search/repositories")
            .query(&[("q", search_query.as_str()), ("per_page", "10")])
            .header(USER_AGENT, "rust-github-bot")
            .send()
            .await?
            .json()
            .await?;

        for repo in response.items {
            repositories.entry(repo.full_name.clone()).or_insert(repo);
        }
    }

    Ok(repositories.into_values().collect())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run -- <repository>");
        return Ok(());
    }

    let repository = &args[1];

    let client = reqwest::Client::new();

    // Direct owner/repository lookup
    if repository.contains('/') {
        let url = format!("https://api.github.com/repos/{}", repository);

        let repo: Repository = client
            .get(url)
            .header(USER_AGENT, "rust-github-bot")
            .send()
            .await?
            .json()
            .await?;

        println!();
        println!("✓ GitHub Repository");
        println!("-------------------");
        println!("Name: {}", repo.name);
        println!("Full name: {}", repo.full_name);
        println!(
            "Description: {}",
            repo.description.as_deref().unwrap_or("No description")
        );
        println!("Stars: {}", repo.stargazers_count);
        println!("Forks: {}", repo.forks_count);
        println!("Open issues: {}", repo.open_issues_count);

        return Ok(());
    }

    // Search GitHub
    let repositories = search_repositories(&client, repository).await?;

    if repositories.is_empty() {
        println!("No repositories found for '{}'.", repository);
        return Ok(());
    }

    match resolve_repository(&repositories, repository) {
        Some(result) => {
            println!();
            println!("✓ Repository found");
            println!("-----------------");
            println!("Name: {}", result.repository.name);
            println!("Full name: {}", result.repository.full_name);
            println!(
                "Description: {}",
                result
                    .repository
                    .description
                    .as_deref()
                    .unwrap_or("No description")
            );
            println!("Stars: {}", result.repository.stargazers_count);
            println!("Forks: {}", result.repository.forks_count);
            println!("Open issues: {}", result.repository.open_issues_count);
            println!();
            println!("Match information");
            println!("-----------------");
            println!("Reason: {}", result.reason.description());
            println!("Score: {}", result.score);
            println!("Confidence: {}%", result.confidence);
        }

        None => {
            println!();
            println!("⚠ Could not confidently identify a repository.");
            println!();
            println!("Top candidates:");

            let mut candidates: Vec<_> = repositories
                .iter()
                .map(|repo| {
                    let (score, reason) = score_repository(repo, repository);

                    (repo, score, reason)
                })
                .filter(|(_, score, _)| *score > 0)
                .collect();

            candidates.sort_by(|a, b| b.1.cmp(&a.1));

            for (index, (repo, score, reason)) in candidates.iter().take(5).enumerate() {
                println!(
                    "{}. {} — Score: {} — {}",
                    index + 1,
                    repo.full_name,
                    score,
                    reason.description()
                );
            }

            println!();
            println!("Try a more specific repository name.");
        }
    }

    Ok(())
}
