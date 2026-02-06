use std::path::Path;

use axum::Router;
use serde::{Deserialize, Serialize};

use crate::AppState;

mod broadcasts;

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    limit: usize,
    offset: usize,
}

#[derive(Serialize, Deserialize)]
pub struct PaginationMeta {
    total: usize,

    #[serde(flatten)]
    pagination: Pagination,

    next: Option<String>,
    prev: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PagedResponse<T> {
    data: Vec<T>,

    #[serde(flatten)]
    meta: PaginationMeta,
}

pub fn router(hls_dir: &Path, state: &AppState) -> Router<AppState> {
    Router::new().merge(broadcasts::router(hls_dir, state))
}
