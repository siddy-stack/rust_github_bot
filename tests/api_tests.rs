use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use rust_bot::api::{AppState, create_router};
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn search_returns_result_for_valid_query() {
    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
    });

    let app = create_router(state);

    let request = Request::builder()
        .uri("/api/search?q=pytest")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn search_rejects_empty_query() {
    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
    });

    let app = create_router(state);

    let request = Request::builder()
        .uri("/api/search?q=")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_returns_not_found_for_unconfident_query() {
    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
    });

    let app = create_router(state);

    let request = Request::builder()
        .uri("/api/search?q=pyest")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
