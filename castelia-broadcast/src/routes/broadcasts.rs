use std::path::Path;

use axum::Router;
use tower_http::services::ServeDir;

pub fn router(hls_dir: &Path) -> Router {
    Router::new().nest_service("/broadcasts", ServeDir::new(hls_dir))
}
