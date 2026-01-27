use std::path::Path;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::error;

use crate::AppState;

#[derive(Deserialize)]
pub struct InitializeBroadcast {
    channel_name: String,
}

pub fn router(hls_dir: &Path) -> Router<AppState> {
    Router::new()
        .route("/broadcasts/initialize", post(initialize_broadcast))
        .nest_service("/broadcasts", ServeDir::new(hls_dir))
}

async fn initialize_broadcast(
    State(state): State<AppState>,
    Json(req): Json<InitializeBroadcast>,
) -> StatusCode {
    if let Err(e) = sqlx::query!(
        r#"INSERT INTO broadcasts (channel_name) VALUES($1)"#,
        req.channel_name
    )
    .execute(&state.db)
    .await
    {
        if let Some(e) = e.as_database_error()
            && e.is_unique_violation()
        {
            error!("Broadcast already exists");
            return StatusCode::BAD_REQUEST;
        }

        error!("Failed to create broadcast: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::CREATED
}
