use std::path::Path;

use axum::Router;
use tower_http::trace::TraceLayer;

mod routes;

pub fn app(hls_dir: &Path) -> Router {
    routes::router(hls_dir).layer(TraceLayer::new_for_http())
}
