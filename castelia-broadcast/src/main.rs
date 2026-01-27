use std::{env, path::PathBuf};

use anyhow::Context;
use castelia_broadcast::app;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Listening on {}", listener.local_addr()?);

    let hls_dir =
        PathBuf::from(env::var("HLS_OUTPUT_DIR").context("Could not find HLS_OUTPUT_DIR")?);

    info!("Serving HLS files from {}", hls_dir.display());

    axum::serve(listener, app(&hls_dir)).await?;
    Ok(())
}
