use std::{env, path::PathBuf};

use anyhow::Context;
use castelia_broadcast::{AppState, app};
use sqlx::postgres::PgPoolOptions;
use tracing::info;

async fn init_state() -> anyhow::Result<AppState> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    Ok(AppState { db: pool })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Listening on {}", listener.local_addr()?);

    let hls_dir =
        PathBuf::from(env::var("HLS_OUTPUT_DIR").context("Could not find HLS_OUTPUT_DIR")?);

    let state = init_state().await?;
    info!("Serving HLS files from {}", hls_dir.display());

    let app = app(state, &hls_dir);
    axum::serve(listener, app).await?;
    Ok(())
}
