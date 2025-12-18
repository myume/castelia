use castelia_rtmp::rtmp::RTMPServer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1935").await?;
    info!("Listening on {}", listener.local_addr()?);

    RTMPServer::new(listener).run().await?;

    Ok(())
}
