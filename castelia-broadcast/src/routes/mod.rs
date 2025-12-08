use axum::{Router, http::StatusCode, routing::get};

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
