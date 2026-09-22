use axum::{
    Json, Router,
    extract::{Query, State},
    response::Html,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{MatchReason, resolve_repository, search_repositories};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Debug, Serialize)]
struct SearchResult {
    name: String,
    full_name: String,
    description: Option<String>,
    stars: u32,
    forks: u32,
    open_issues: u32,
    score: u32,
    confidence: u32,
    reason: String,
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResult>, (axum::http::StatusCode, String)> {
    if params.q.trim().is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Query cannot be empty".to_string(),
        ));
    }

    let repositories = search_repositories(&state.client, &params.q)
        .await
        .map_err(|error| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
            )
        })?;

    let result = resolve_repository(&repositories, &params.q).ok_or((
        axum::http::StatusCode::NOT_FOUND,
        "Could not confidently identify a repository".to_string(),
    ))?;

    Ok(Json(SearchResult {
        name: result.repository.name.clone(),
        full_name: result.repository.full_name.clone(),
        description: result.repository.description.clone(),
        stars: result.repository.stargazers_count,
        forks: result.repository.forks_count,
        open_issues: result.repository.open_issues_count,
        score: result.score,
        confidence: result.confidence,
        reason: match result.reason {
            MatchReason::ExactRepository => "Exact owner/repository match",
            MatchReason::ExactName => "Exact repository name",
            MatchReason::Prefix => "Repository name starts with query",
            MatchReason::Contains => "Repository name contains query",
            MatchReason::DescriptionPhrase => "Exact phrase found in description",
            MatchReason::TokenMatch => "Strong token match",
            MatchReason::Abbreviation => "Abbreviation match",
            MatchReason::Fuzzy => "Fuzzy name match",
        }
        .to_string(),
    }))
}

pub async fn home() -> Html<&'static str> {
    Html(include_str!("../Static/index.html"))
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/api/search", get(search))
        .with_state(state)
}
