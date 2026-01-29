use std::path::Path;

use axum::Router;

use crate::AppState;

mod broadcasts;

pub fn router(hls_dir: &Path, state: &AppState) -> Router<AppState> {
    Router::new().merge(broadcasts::router(hls_dir, state))
}
