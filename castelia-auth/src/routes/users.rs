use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/{username}", get(get_user))
}

async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>, StatusCode> {
    match sqlx::query_as!(
        User,
        "SELECT id, username FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
