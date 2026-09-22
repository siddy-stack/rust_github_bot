use serde::Deserialize;

const CONFIDENCE_EXACT_REPOSITORY: u32 = 99;
const CONFIDENCE_EXACT_NAME: u32 = 97;
const CONFIDENCE_ABBREVIATION: u32 = 92;
const CONFIDENCE_MULTIWORD_NAME: u32 = 87;
const CONFIDENCE_PREFIX: u32 = 78;
const CONFIDENCE_CONTAINS: u32 = 65;
const CONFIDENCE_TOKEN_MATCH: u32 = 55;
const CONFIDENCE_PARTIAL_TOKEN: u32 = 45;
const CONFIDENCE_FUZZY: u32 = 35;
const CONFIDENCE_WEAK: u32 = 20;
const CONFIDENCE_UNIQUENESS_BONUS: f64 = 5.0;
const EXACT_REPOSITORY_SCORE: u32 = 5000;
const EXACT_NAME_SCORE: u32 = 4000;
const MULTIWORD_NAME_SCORE: u32 = 3000;
const ABBREVIATION_SCORE: u32 = 3500;
const PREFIX_SCORE: u32 = 2500;
const CONTAINS_SCORE: u32 = 1500;
const DESCRIPTION_PHRASE_SCORE: u32 = 1200;
const DESCRIPTION_ABBREVIATION_SCORE: u32 = 1100;
const TOKEN_MATCH_SCORE: u32 = 1000;
const PARTIAL_TOKEN_SCORE: u32 = 700;
const FUZZY_SCORE: u32 = 600;
const EXACT_MATCH_THRESHOLD: u32 = 4000;
const SEMANTIC_MATCH_THRESHOLD: u32 = 1000;
const EXACT_NAME_RANKING_BONUS: u64 = 100_000;
const CONTAINS_NAME_RANKING_BONUS: u64 = 50_000;
const ABBREVIATION_RANKING_BONUS: u64 = 75_000;
const DESCRIPTION_PHRASE_RANKING_BONUS: u64 = 10_000;
const DESCRIPTION_ABBREVIATION_RANKING_BONUS: u64 = 25_000;
const RANKING_GAP_THRESHOLD: u64 = 50_000;

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchReason {
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
    pub fn description(&self) -> &'static str {
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

#[derive(Debug)]
pub struct MatchResult<'a> {
    pub repository: &'a Repository,
    pub score: u32,
    pub confidence: u32,
    pub reason: MatchReason,
}

pub fn normalize_text(text: &str) -> String {
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

pub fn expand_query(query: &str) -> Vec<String> {
    let normalized = normalize_text(query);

    let mut queries = vec![normalized.clone()];

    // "VS Code" -> "vscode"
    if normalized == "vs code" {
        queries.push("vscode".to_string());
    }

    // "Visual Studio Code" -> "vscode"
    if normalized == "visual studio code" {
        queries.push("vscode".to_string());
    }

    // Search multi-word queries as an exact phrase too.
    if normalized.split_whitespace().count() > 1 {
        queries.push(format!("\"{}\"", normalized));
    }

    queries
}

pub fn score_repository(repo: &Repository, query: &str) -> (u32, MatchReason) {
    let query = normalize_text(query);
    if query.is_empty() {
        return (0, MatchReason::Fuzzy);
    }
    let name = normalize_text(&repo.name);
    let full_name = normalize_text(&repo.full_name);

    let description = normalize_text(repo.description.as_deref().unwrap_or(""));

    // Exact owner/repository match
    if full_name == query {
        return (EXACT_REPOSITORY_SCORE, MatchReason::ExactRepository);
    }

    // Exact repository name
    if name == query {
        if query.split_whitespace().count() == 1 {
            return (EXACT_NAME_SCORE, MatchReason::ExactName);
        }

        return (MULTIWORD_NAME_SCORE, MatchReason::ExactName);
    }

    // Abbreviation match
    if abbreviation_name_matches(&query, &name) {
        return (ABBREVIATION_SCORE, MatchReason::Abbreviation);
    }

    // Repository name starts with query
    if name.starts_with(&query) {
        return (PREFIX_SCORE, MatchReason::Prefix);
    }

    // Repository name contains query
    if name.contains(&query) {
        return (CONTAINS_SCORE, MatchReason::Contains);
    }

    // Exact phrase appears in description
    if description.contains(&query) {
        return (DESCRIPTION_PHRASE_SCORE, MatchReason::DescriptionPhrase);
    }

    // Abbreviation in description
    if abbreviation_matches(&query, &description) {
        return (DESCRIPTION_ABBREVIATION_SCORE, MatchReason::Abbreviation);
    }

    // Token matching
    //
    // Token matching should use actual word relationships.
    // Fuzzy similarity is handled separately below.
    let query_words: Vec<&str> = query.split_whitespace().collect();

    if !query_words.is_empty() {
        let matched_words = query_words
            .iter()
            .filter(|word| {
                name.split_whitespace()
                    .any(|name_word| name_word == **word || name_word.starts_with(**word))
                    || description.split_whitespace().any(|description_word| {
                        description_word == **word || description_word.starts_with(**word)
                    })
            })
            .count();

        if matched_words > 0 {
            let percentage = (matched_words * 100) / query_words.len();

            if percentage == 100 {
                return (TOKEN_MATCH_SCORE, MatchReason::TokenMatch);
            }

            if percentage >= 50 {
                return (PARTIAL_TOKEN_SCORE, MatchReason::TokenMatch);
            }
        }
    }

    // Fuzzy repository-name matching
    let fuzzy_score = similarity(&name, &query);

    if fuzzy_score >= FUZZY_SCORE {
        return (fuzzy_score, MatchReason::Fuzzy);
    }

    (0, MatchReason::Fuzzy)
}

fn ranking_score(repo: &Repository, query: &str) -> u64 {
    let query = normalize_text(query);
    let name = normalize_text(&repo.name);

    let description = normalize_text(repo.description.as_deref().unwrap_or(""));

    // Stars are only a small tiebreaker.
    let mut score = repo.stargazers_count as u64;

    // Exact repository name
    if name == query {
        score += EXACT_NAME_RANKING_BONUS;
    } else if name.contains(&query) {
        score += CONTAINS_NAME_RANKING_BONUS;
    }

    // Abbreviation match
    if abbreviation_name_matches(&query, &name) {
        score += ABBREVIATION_RANKING_BONUS;
    }

    // Description phrase
    if description.contains(&query) {
        score += DESCRIPTION_PHRASE_RANKING_BONUS;
    }

    // Abbreviation in description
    if abbreviation_matches(&query, &description) {
        score += DESCRIPTION_ABBREVIATION_RANKING_BONUS;
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
        5000.. => CONFIDENCE_EXACT_REPOSITORY,
        4000..=4999 => CONFIDENCE_EXACT_NAME,
        3500..=3999 => CONFIDENCE_ABBREVIATION,
        3000..=3499 => CONFIDENCE_MULTIWORD_NAME,
        2500..=2999 => CONFIDENCE_PREFIX,
        1500..=2499 => CONFIDENCE_CONTAINS,
        1000..=1499 => CONFIDENCE_TOKEN_MATCH,
        700..=999 => CONFIDENCE_PARTIAL_TOKEN,
        600..=699 => CONFIDENCE_FUZZY,
        _ => CONFIDENCE_WEAK,
    };

    let uniqueness_bonus = (gap_ratio * CONFIDENCE_UNIQUENESS_BONUS) as u32;

    (base + uniqueness_bonus).min(100)
}

pub fn resolve_repository<'a>(
    repositories: &'a [Repository],
    query: &str,
) -> Option<MatchResult<'a>> {
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

    // --------------------------------------------------------
    // Exact matches
    // --------------------------------------------------------

    if best.1 >= EXACT_MATCH_THRESHOLD && best.1 > second_score {
        return Some(MatchResult {
            repository: best.0,
            score: best.1,
            confidence,
            reason: best.2,
        });
    }

    // --------------------------------------------------------
    // Strong semantic matches
    // --------------------------------------------------------

    if best.1 >= SEMANTIC_MATCH_THRESHOLD {
        let ranking_gap = best
            .3
            .saturating_sub(scored.get(1).map(|result| result.3).unwrap_or(0));

        if best.1 > second_score || ranking_gap >= RANKING_GAP_THRESHOLD {
            return Some(MatchResult {
                repository: best.0,
                score: best.1,
                confidence,
                reason: best.2,
            });
        }
    }

    // --------------------------------------------------------
    // Strong fuzzy matches
    // --------------------------------------------------------

    if matches!(best.2, MatchReason::Fuzzy) && best.1 >= FUZZY_SCORE && best.1 > second_score {
        return Some(MatchResult {
            repository: best.0,
            score: best.1,
            confidence,
            reason: best.2,
        });
    }

    None
}
