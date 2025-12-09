use castelia_rtmp::rtmp::RTMPSever;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("Listening on {}", listener.local_addr()?);

    RTMPSever::new(listener).run().await?;

    Ok(())
}
