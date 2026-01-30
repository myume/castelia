use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use tracing::error;

use crate::{
    AppState,
    routes::{Claims, RefreshClaims, TokenType, login::AccessTokenResponse, users::User},
};

pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

pub fn generate_access_token(
    user: User,
    encryption_key: &[u8],
) -> Result<String, jsonwebtoken::errors::Error> {
    #[allow(clippy::expect_used)]
    let exp = Utc::now()
        .checked_add_signed(TimeDelta::minutes(30))
        .expect("Invalid expiration date");
    let claims = Claims {
        sub: user.id,
        exp: exp.timestamp(),
        username: user.username,
        token_type: TokenType::Access,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(encryption_key),
    )
}

pub async fn jwt(claim: Claims) -> Json<Claims> {
    Json(claim)
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, StatusCode> {
    let Some(cookie) = jar.get(REFRESH_TOKEN_COOKIE) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(token_data) = jsonwebtoken::decode::<RefreshClaims>(
        cookie.value(),
        &DecodingKey::from_secret(state.encryption_key.as_ref()),
        &Validation::default(),
    ) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(user) = sqlx::query_as!(
        User,
        "SELECT id, username FROM users WHERE id = $1",
        token_data.claims.sub
    )
    .fetch_optional(&state.db)
    .await
    else {
        error!("Failed to fetch user");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if let Some(user) = user {
        let Ok(access_token) = generate_access_token(user, &state.encryption_key) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        return Ok(Json(AccessTokenResponse { access_token }));
    }

    Err(StatusCode::UNAUTHORIZED)
}
