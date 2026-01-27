use std::path::Path;

use axum::Router;
use sqlx::Pool;
use tower_http::trace::TraceLayer;

mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<sqlx::Postgres>,
}

pub fn app(state: AppState, hls_dir: &Path) -> Router {
    routes::router(hls_dir)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
