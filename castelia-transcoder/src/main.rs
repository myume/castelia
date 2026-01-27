use std::{env, path::PathBuf};

use anyhow::{Context, anyhow};
use castelia_events::stream_events::STREAM_EVENT_KEY;
use redis::Commands;
use tracing::info;

use crate::transcoder::TranscoderService;

mod transcoder;

const TRANSCODER_GROUP: &str = "transcoders";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = redis::Client::open(env::var("REDIS_URL").context("REDIS_URL is missing")?)
        .context("Failed to initialize redis client")?;

    let mut conn = client.get_connection()?;

    let result: redis::RedisResult<()> =
        conn.xgroup_create_mkstream(STREAM_EVENT_KEY, TRANSCODER_GROUP, "0");
    match result {
        Ok(_) => info!("Consumer group created successfully."),
        Err(e) if e.code() == Some("BUSYGROUP") => {
            info!("Consumer group already exists, skipping creation.");
        }
        Err(e) => return Err(anyhow!(e)),
    };

    let output_dir = env::var("HLS_OUTPUT_DIR").unwrap_or("./".to_string());

    let mut transcoder_service = TranscoderService::new(&PathBuf::from(output_dir), client);
    transcoder_service.start().await
}
