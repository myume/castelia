use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Connection;
use tower_http::services::ServeDir;
use tracing::{debug, error};

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Broadcast {
    channel_name: String,
    title: String,
    start_time: Option<DateTime<Utc>>,
    status: String,
    private: bool,
}

#[derive(Debug, thiserror::Error)]
enum GetBroadcastError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("Failed to initialize broadcast: {0}")]
    InitializeBroadcast(sqlx::Error),

    #[error("Could not fetch broadcast: {0}")]
    FetchBroadcast(sqlx::Error),
}

impl IntoResponse for GetBroadcastError {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetBroadcastError::InitializeBroadcast(error)
            | GetBroadcastError::FetchBroadcast(error)
            | GetBroadcastError::Database(error) => {
                error!("{error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct UpdateBroadcast {
    title: Option<String>,
    status: Option<String>,
    private: Option<bool>,
}

pub fn router(hls_dir: &std::path::Path) -> Router<AppState> {
    Router::new()
        .route(
            "/broadcasts/{channel}",
            get(get_broadcast).patch(update_broadcast),
        )
        .nest_service("/broadcasts/hls", ServeDir::new(hls_dir))
}

async fn update_broadcast(
    State(state): State<AppState>,
    Path(channel_name): Path<String>,
    Json(req): Json<UpdateBroadcast>,
) -> StatusCode {
    if let Err(e) = sqlx::query!(
        "UPDATE broadcasts SET title = $1, status = $2, private = $3 
        WHERE channel_name = $4",
        req.title,
        req.status,
        req.private,
        channel_name,
    )
    .execute(&state.db)
    .await
    {
        error!("Failed to update broadcast metadata: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn get_broadcast(
    State(state): State<AppState>,
    Path(channel_name): Path<String>,
) -> Result<Json<Broadcast>, GetBroadcastError> {
    let mut conn = state.db.acquire().await?;
    let mut tx = conn.begin().await?;

    let fetch_result = sqlx::query_as!(
        Broadcast,
        "SELECT channel_name, title, start_time, status, private FROM broadcasts WHERE channel_name = $1",
        channel_name
    ).fetch_optional(&mut *tx).await;

    let broadcast = match fetch_result {
        Ok(Some(broadcast)) => broadcast,
        Ok(None) => {
            debug!("broadcast not found for {channel_name}, initializing...");
            sqlx::query_as!(
                Broadcast,
                "INSERT INTO broadcasts (channel_name) VALUES($1)
                 RETURNING channel_name, title, start_time, status, private",
                channel_name
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(GetBroadcastError::InitializeBroadcast)?
        }
        Err(e) => return Err(GetBroadcastError::FetchBroadcast(e)),
    };

    tx.commit().await?;

    Ok(Json(broadcast))
}
