use std::{env, path::PathBuf};

use anyhow::Context;
use castelia_broadcast::{AppState, app, event_handler::handle_events};
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

async fn init_state() -> anyhow::Result<AppState> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    let auth_url = env::var("AUTH_URL")?;
    let client = reqwest::Client::new();

    Ok(AppState {
        db: pool,
        auth_url,
        client,
    })
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

    let db = state.db.clone();
    tokio::spawn(async move {
        if let Err(e) = handle_events(db).await {
            error!("Event handler crashed: {e}")
        }
    });

    let app = app(state, &hls_dir);
    axum::serve(listener, app).await?;
    Ok(())
}
