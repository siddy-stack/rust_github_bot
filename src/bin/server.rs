use rust_bot::api::{AppState, create_router};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
    });

    let app = create_router(state);

    let address = "127.0.0.1:3000";

    println!("Server running at http://{}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind server");

    axum::serve(listener, app).await.expect("Server failed");
}
