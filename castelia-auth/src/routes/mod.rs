use axum::{
    Json, RequestPartsExt, Router,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::AppState;

mod login;
mod signup;
pub mod stream_key;
pub mod users;

pub enum AuthError {
    InvalidToken,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub sub: uuid::Uuid,
    pub username: String,
}

impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(|_| AuthError::InvalidToken)?;
        let token_data = jsonwebtoken::decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(state.encryption_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/login", post(login::login))
        .route("/signup", post(signup::signup))
        .route("/streamkey", post(stream_key::verify_streamkey))
        .route("/streamkey", get(stream_key::get_streamkey))
        .route("/jwt", get(jwt))
        .nest("/user", users::router())
        .with_state(state)
}

async fn jwt(claim: Claims) -> Json<Claims> {
    Json(claim)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
