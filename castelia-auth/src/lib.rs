use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use sqlx::Pool;
use tower_http::trace::TraceLayer;

mod routes;

pub struct AppState {
    pub db: Pool<sqlx::Postgres>,
    pub cipher: Aes256Gcm,
}

pub fn app(state: AppState) -> axum::Router {
    routes::router(Arc::new(state)).layer(TraceLayer::new_for_http())
}
