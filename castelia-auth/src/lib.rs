use std::sync::Arc;

use sqlx::Pool;
use tower_http::trace::TraceLayer;

mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<sqlx::Postgres>,
    pub encryption_key: Vec<u8>,
}

pub fn app(state: AppState) -> axum::Router {
    routes::router(Arc::new(state)).layer(TraceLayer::new_for_http())
}
