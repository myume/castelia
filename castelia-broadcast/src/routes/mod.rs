use std::path::Path;

use axum::Router;

mod broadcasts;

pub fn router(hls_dir: &Path) -> Router {
    Router::new().merge(broadcasts::router(hls_dir))
}
