use std::{collections::HashMap, env};

use anyhow::{Context, anyhow};
use castelia_events::stream_events::{STREAM_EVENT_KEY, StreamEvent};
use redis::{
    Commands,
    streams::{StreamReadOptions, StreamReadReply},
};
use tokio::process::Command;
use tracing::{info, trace, warn};

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

    let opts = StreamReadOptions::default()
        .group(TRANSCODER_GROUP, "transcoder_group")
        .count(10)
        .block(5000);

    let mut ffmpeg_processes = HashMap::new();

    loop {
        let reply: StreamReadReply = conn.xread_options(&[STREAM_EVENT_KEY], &[">"], &opts)?;

        for stream in reply.keys {
            for entry in stream.ids {
                trace!("{:?}", &entry);
                let Some(payload) = entry.map.get("data") else {
                    warn!("Missing payload on stream event");
                    continue;
                };
                let payload: String = redis::from_redis_value(payload.clone())?;
                let event: StreamEvent = serde_json::from_str(&payload)?;
                match event {
                    StreamEvent::Start {
                        stream_id,
                        rtmp_url,
                    } => {
                        info!("Starting transcoder for stream: {stream_id}");

                        let mut command = Command::new("ffmpeg");
                        command
                            .arg("-loglevel")
                            .arg("error")
                            .arg("-i")
                            .arg(format!("{rtmp_url}/{stream_id}"))
                            .arg("-c:v")
                            .arg("libx264")
                            .arg("-c:a")
                            .arg("aac")
                            .arg("-f")
                            .arg("hls")
                            .arg("stream.m3m8");

                        let handle = command.spawn().context("Failed to spawn ffmpeg process")?;
                        ffmpeg_processes.insert(stream_id, handle);
                    }
                    StreamEvent::Stop { stream_id } => {
                        info!("Stopping transcoder for stream: {stream_id}");
                        let Some(mut handle) = ffmpeg_processes.remove(&stream_id) else {
                            warn!("transcoder has already been stopped.");
                            continue;
                        };
                        handle
                            .kill()
                            .await
                            .context("Could not kill ffmpeg process")?;
                        info!("Successfully stopped transcoding for stream: {stream_id}")
                    }
                };

                let _: () = conn.xack(STREAM_EVENT_KEY, TRANSCODER_GROUP, &[&entry.id])?;
            }
        }
    }
}
