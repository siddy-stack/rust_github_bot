use reqwest::header::USER_AGENT;
use std::env;

use rust_bot::Repository;

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
    let repositories = rust_bot::search_repositories(&client, repository).await?;

    if repositories.is_empty() {
        println!("No repositories found for '{}'.", repository);
        return Ok(());
    }

    match rust_bot::resolve_repository(&repositories, repository) {
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
                    let (score, reason) = rust_bot::score_repository(repo, repository);
                    (repo, score, reason)
                })
                .filter(|(_, score, _)| *score > 0)
                .collect();

            candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.1));

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
