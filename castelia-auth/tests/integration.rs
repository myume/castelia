use aes_gcm::{Aes256Gcm, KeyInit};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use castelia_auth::{AppState, app};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test(migrations = "../migrations")]
fn test_signup(pool: PgPool) {
    tracing_subscriber::fmt::init();

    let state = AppState {
        db: pool.clone(),
        cipher: Aes256Gcm::new(&[0; 32].into()),
    };

    let username = "test";
    let password = "password";
    let email = "email@email.com";

    let response = app(state)
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri("/signup")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": password,
                        "email": email
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let user = sqlx::query!(
        "SELECT username, password, email FROM users WHERE username = $1",
        username
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(user.username, username);
    assert_eq!(user.email, email);
    assert!(
        Argon2::default()
            .verify_password(
                password.as_bytes(),
                &PasswordHash::new(&user.password).unwrap(),
            )
            .is_ok()
    );
}
