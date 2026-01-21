use std::env;

use aes_gcm::{Aes256Gcm, KeyInit};
use castelia_auth::{AppState, app};
use sqlx::postgres::PgPoolOptions;
use tracing::info;

async fn init_state() -> anyhow::Result<AppState> {
    let db_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5) // make this dynamically configurable?
        .connect(&db_url)
        .await?;

    let encryption_key = hex::decode(env::var("ENCRYPTION_KEY")?)?;
    let cipher = Aes256Gcm::new(encryption_key.as_slice().into());

    Ok(AppState { db: pool, cipher })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::fmt::init();

    let state = init_state().await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app(state)).await?;

    Ok(())
}
