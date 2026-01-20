use std::sync::Arc;

use sqlx::{Pool, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;
use tracing::info;

mod routes;

struct AppState {
    db_pool: Pool<sqlx::Postgres>,
}

async fn init_state() -> anyhow::Result<AppState> {
    let pool = PgPoolOptions::new()
        .max_connections(5) // make this dynamically configurable?
        .connect("postgres://postgres:password@localhost/test")
        .await?;

    Ok(AppState { db_pool: pool })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = init_state().await?;

    let app = routes::router(Arc::new(state)).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
