use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use castelia_auth::{AppState, app};
use http_body_util::BodyExt;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test(migrations = "../migrations")]
fn test_signup_success(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
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

#[sqlx::test(migrations = "../migrations")]
fn test_login_success(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let username = "test";
    let password = "password";
    let email = "email@email.com";

    app(state.clone())
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

    let response = app(state)
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri("/login")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": password,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../migrations")]
fn test_login_wrong_password(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let username = "test";
    let password = "password";
    let email = "email@email.com";

    app(state.clone())
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

    let response = app(state)
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri("/login")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": "wrong password",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Invalid username or password.");
}

#[sqlx::test(migrations = "../migrations")]
fn test_login_wrong_username(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let username = "test";
    let password = "password";
    let email = "email@email.com";

    app(state.clone())
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

    let response = app(state)
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri("/login")
                .body(Body::from(
                    json!({
                        "username": "some_random_user",
                        "password": "wrong password",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Invalid username or password.");
}
