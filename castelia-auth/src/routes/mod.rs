use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use crate::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/login", post(login))
        .route("/signup", post(signup))
        .route("/verify", post(verify_streamkey))
        .with_state(state)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn signup(State(state): State<Arc<AppState>>) {}

async fn login() {}

async fn verify_streamkey() {}
