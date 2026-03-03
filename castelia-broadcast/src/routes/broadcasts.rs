use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use castelia_auth::routes::Claims;
use chrono::{DateTime, Utc};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use sqlx::Connection;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::{debug, error};

use crate::{
    AppState,
    routes::{PagedResponse, Pagination, PaginationMeta},
};

#[derive(Debug, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "stream_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum StreamStatus {
    Offline,
    Unpublished,
    Published,
}

#[derive(Serialize, Deserialize)]
pub struct Broadcast {
    channel_name: String,
    title: String,
    start_time: Option<DateTime<Utc>>,
    status: StreamStatus,
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

    #[error("User with the username {0} could not be found")]
    UserNotFound(String),

    #[error("Failed to retrieve user")]
    UserUnretrievable,
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
            GetBroadcastError::UserNotFound(_) => StatusCode::NOT_FOUND.into_response(),
            GetBroadcastError::UserUnretrievable => StatusCode::BAD_GATEWAY.into_response(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateBroadcast {
    title: Option<String>,
    private: Option<bool>,
}

enum GetUserError {
    Unretrievable,
    NotFound(String),
}
async fn user_exists(
    auth_url: &str,
    channel_name: &str,
    client: &reqwest::Client,
) -> Result<(), GetUserError> {
    let res = client
        .get(format!("{auth_url}/user/{channel_name}"))
        .send()
        .await
        .map_err(|_| GetUserError::Unretrievable)?;
    if res.status().is_server_error() {
        error!("Auth service returned error response");
        return Err(GetUserError::Unretrievable);
    }
    if res.status().is_client_error() {
        return Err(GetUserError::NotFound(channel_name.to_string()));
    }
    Ok(())
}

pub fn router(hls_dir: &std::path::Path, state: &AppState) -> Router<AppState> {
    let hls_server = ServiceBuilder::new()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            validate_broadcast,
        ))
        .service(ServeDir::new(hls_dir));

    Router::new()
        .route("/broadcasts/list", get(list_broadcasts))
        .route("/broadcasts/{channel}/publish", post(start_broadcast))
        .route("/broadcasts/{channel}/unpublish", post(stop_broadcast))
        .route(
            "/broadcasts/{channel}",
            get(get_broadcast).patch(update_broadcast),
        )
        .nest_service("/broadcasts/hls", hls_server)
}

async fn validate_broadcast(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let parts = request.uri().path().split("/").collect::<Vec<&str>>();
    let channel_name = parts.get(1).ok_or(StatusCode::NOT_FOUND)?;

    let broadcast = sqlx::query!(
        r#"SELECT status as "status: StreamStatus", private FROM broadcasts WHERE channel_name = $1"#,
        channel_name
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to retrieve broadcast: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    if broadcast.private || broadcast.status != StreamStatus::Published {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

async fn validate_jwt(
    auth_url: &str,
    authorization: &HeaderValue,
    client: &reqwest::Client,
) -> Result<Claims, StatusCode> {
    let res = client
        .get(format!("{auth_url}/jwt"))
        .header(AUTHORIZATION, authorization)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send auth request");
            e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        })?
        .error_for_status()
        .map_err(|e| e.status().unwrap_or(StatusCode::UNAUTHORIZED))?;

    let claims = res.json().await.map_err(|e| {
        error!("Failed to deserialize claims: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(claims)
}

async fn authorize_operation(
    auth_url: &str,
    channel_name: &str,
    headers: &HeaderMap,
    client: &reqwest::Client,
) -> StatusCode {
    let Some(authorization) = headers.get(AUTHORIZATION) else {
        return StatusCode::UNAUTHORIZED;
    };

    match validate_jwt(auth_url, authorization, client).await {
        Ok(claims) => {
            if claims.username != channel_name {
                return StatusCode::UNAUTHORIZED;
            }
        }
        Err(status) => return status,
    }

    StatusCode::OK
}

async fn update_broadcast(
    State(state): State<AppState>,
    Path(channel_name): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateBroadcast>,
) -> Response {
    let status = authorize_operation(&state.auth_url, &channel_name, &headers, &state.client).await;
    if !status.is_success() {
        return status.into_response();
    }

    let broadcast = match get_broadcast(State(state.clone()), Path(channel_name.clone())).await {
        Ok(Json(broadcast)) => broadcast,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = sqlx::query!(
        "UPDATE broadcasts SET title = $1, private = $2 
        WHERE channel_name = $3",
        req.title.unwrap_or(broadcast.title),
        req.private.unwrap_or(broadcast.private),
        channel_name,
    )
    .execute(&state.db)
    .await
    {
        error!("Failed to update broadcast metadata: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
}

async fn get_broadcast(
    State(state): State<AppState>,
    Path(channel_name): Path<String>,
) -> Result<Json<Broadcast>, GetBroadcastError> {
    if let Err(e) = user_exists(&state.auth_url, &channel_name, &state.client).await {
        return Err(match e {
            GetUserError::Unretrievable => GetBroadcastError::UserUnretrievable,
            GetUserError::NotFound(username) => GetBroadcastError::UserNotFound(username),
        });
    }

    let mut conn = state.db.acquire().await?;
    let mut tx = conn.begin().await?;

    let fetch_result = sqlx::query_as!(
        Broadcast,
        r#"SELECT channel_name, title, start_time, status as "status: StreamStatus", private FROM broadcasts WHERE channel_name = $1"#,
        channel_name
    ).fetch_optional(&mut *tx).await;

    let broadcast = match fetch_result {
        Ok(Some(broadcast)) => broadcast,
        Ok(None) => {
            debug!("broadcast not found for {channel_name}, initializing...");
            sqlx::query_as!(
                Broadcast,
                r#"INSERT INTO broadcasts (channel_name) VALUES($1)
                 RETURNING channel_name, title, start_time, status as "status: StreamStatus", private"#,
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

async fn start_broadcast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(channel_name): Path<String>,
) -> impl IntoResponse {
    let status = authorize_operation(&state.auth_url, &channel_name, &headers, &state.client).await;
    if !status.is_success() {
        return (status, "Not authorized");
    }

    let Ok(broadcast) = sqlx::query!(
        r#"SELECT status as "status: StreamStatus" FROM broadcasts WHERE channel_name = $1"#,
        channel_name
    )
    .fetch_one(&state.db)
    .await
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update");
    };

    if broadcast.status == StreamStatus::Offline {
        return (StatusCode::BAD_REQUEST, "Stream is offline");
    }

    if let Err(e) = sqlx::query!(
        "UPDATE broadcasts SET status = 'published', start_time = $1 WHERE channel_name = $2",
        Utc::now(),
        channel_name
    )
    .execute(&state.db)
    .await
    {
        error!("Failed to update broadcast status: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update broadcast status",
        );
    };

    (StatusCode::OK, "Success")
}

async fn stop_broadcast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(channel_name): Path<String>,
) -> impl IntoResponse {
    let status = authorize_operation(&state.auth_url, &channel_name, &headers, &state.client).await;
    if !status.is_success() {
        return (status, "Not authorized");
    }

    let Ok(broadcast) = sqlx::query!(
        r#"SELECT status as "status: StreamStatus" FROM broadcasts WHERE channel_name = $1"#,
        channel_name
    )
    .fetch_one(&state.db)
    .await
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update");
    };

    if broadcast.status != StreamStatus::Published {
        return (StatusCode::BAD_REQUEST, "Stream is already unpublished");
    }

    if let Err(e) = sqlx::query!(
        "UPDATE broadcasts
        SET status = CASE 
            WHEN status <> 'offline' THEN 'unpublished'
            ELSE status
        END, 
        start_time = NULL WHERE channel_name = $1",
        channel_name
    )
    .execute(&state.db)
    .await
    {
        error!("Failed to update broadcast status: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update broadcast status",
        );
    };

    (StatusCode::OK, "Success")
}

async fn list_broadcasts(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PagedResponse<Broadcast>>, GetBroadcastError> {
    let total = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM broadcasts"#)
        .fetch_one(&state.db)
        .await?
        .count as usize;

    let data = sqlx::query_as!(
        Broadcast,
        r#"SELECT channel_name, status as "status: StreamStatus", title, start_time, private 
           FROM broadcasts
           WHERE private = false AND status = 'published'
           LIMIT $1 OFFSET $2"#,
        pagination.limit as i64,
        pagination.offset as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(GetBroadcastError::FetchBroadcast)?;

    let next = if total > pagination.offset + pagination.limit {
        Some(format!(
            "/broadcasts?offset={}&limit={}",
            pagination.offset + pagination.limit,
            pagination
                .limit
                .min(total - pagination.offset - pagination.limit)
        ))
    } else {
        None
    };

    let prev = if pagination.offset > 0 {
        Some(format!(
            "/broadcasts?offset={}&limit={}",
            pagination.offset.saturating_sub(pagination.limit),
            pagination.limit
        ))
    } else {
        None
    };

    let response = PagedResponse {
        data,
        meta: PaginationMeta {
            total,
            pagination,
            next,
            prev,
        },
    };

    Ok(Json(response))
}
