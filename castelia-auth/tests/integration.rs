use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    response::Response,
};
use castelia_auth::{AppState, app};
use http_body_util::BodyExt;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

#[derive(Debug, Clone)]
struct User {
    username: String,
    email: String,
    password: String,
}

#[allow(clippy::unwrap_used)]
async fn signup(user: &User, state: &AppState) -> Response<Body> {
    app(state.clone())
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri("/signup")
                .body(Body::from(
                    json!({
                        "username": user.username,
                        "password": user.password,
                        "email": user.email
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[allow(clippy::unwrap_used)]
async fn login(user: &User, state: &AppState) -> Response<Body> {
    app(state.clone())
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri("/login")
                .body(Body::from(
                    json!({
                        "username": user.username,
                        "password": user.password,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[allow(clippy::unwrap_used)]
async fn get_stream_key(access_token: &str, state: &AppState) -> Response<Body> {
    app(state.clone())
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .header(
                    http::header::AUTHORIZATION,
                    format!("Bearer {}", access_token),
                )
                .uri("/streamkey")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../migrations")]
fn test_signup_success(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let expected_user = User {
        username: "test".into(),
        password: "password".into(),
        email: "email@email.com".into(),
    };

    let response = signup(&expected_user, &state).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let actual_user = sqlx::query!(
        "SELECT username, password, email FROM users WHERE username = $1",
        expected_user.username
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(actual_user.username, expected_user.username);
    assert_eq!(actual_user.email, expected_user.email);
    assert!(
        Argon2::default()
            .verify_password(
                expected_user.password.as_bytes(),
                &PasswordHash::new(&actual_user.password).unwrap(),
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

    let expected_user = User {
        username: "test".into(),
        password: "password".into(),
        email: "email@email.com".into(),
    };

    signup(&expected_user, &state).await;
    let response = login(&expected_user, &state).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../migrations")]
fn test_login_wrong_password(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let expected_user = User {
        username: "test".into(),
        password: "password".into(),
        email: "email@email.com".into(),
    };

    signup(&expected_user, &state).await;

    let mut wrong_user = expected_user.clone();
    wrong_user.password = "wrong_password".into();
    let response = login(&wrong_user, &state).await;

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

    let expected_user = User {
        username: "test".into(),
        password: "password".into(),
        email: "email@email.com".into(),
    };

    signup(&expected_user, &state).await;

    let mut wrong_user = expected_user.clone();
    wrong_user.username = "wrong_username".into();
    let response = login(&wrong_user, &state).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Invalid username or password.");
}

#[sqlx::test(migrations = "../migrations")]
fn test_get_streamkey(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let user = User {
        username: "test".into(),
        password: "password".into(),
        email: "email@email.com".into(),
    };
    signup(&user, &state).await;
    let login = login(&user, &state).await;

    let body = login.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let response = get_stream_key(body["access_token"].as_str().unwrap(), &state).await;

    assert_eq!(response.status(), StatusCode::OK);

    let stream_key = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(
        stream_key.starts_with("cast_"),
        "Bad stream key {stream_key}"
    )
}

#[sqlx::test(migrations = "../migrations")]
fn test_get_streamkey_unauthorized(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let response = get_stream_key("bad_access_token", &state).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../migrations")]
fn test_get_streamkey_no_jwt(pool: PgPool) {
    let state = AppState {
        db: pool.clone(),
        encryption_key: vec![0; 32],
    };

    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/streamkey")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
