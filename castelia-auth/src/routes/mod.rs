use std::sync::Arc;

use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};

use crate::AppState;

mod signup;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/login", post(login))
        .route("/signup", post(signup::signup))
        .route("/streamkey", post(verify_streamkey))
        .route("/streamkey", get(get_streamkey))
        .with_state(state)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn login() {}

async fn get_streamkey() {}

async fn verify_streamkey() {}
