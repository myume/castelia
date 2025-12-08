use axum::{Router, routing::get};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listing on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
