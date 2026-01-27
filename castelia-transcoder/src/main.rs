use std::{collections::HashMap, env, path::PathBuf};

use anyhow::{Context, anyhow};
use castelia_events::stream_events::{STREAM_EVENT_KEY, StreamEvent};
use redis::{
    Commands,
    streams::{StreamReadOptions, StreamReadReply},
};
use tokio::{
    fs::{create_dir, remove_dir},
    process::{Child, Command},
};
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
        .block(5000);

    let mut ffmpeg_processes: HashMap<String, Child> = HashMap::new();

    let output_dir = env::var("HLS_OUTPUT_DIR").unwrap_or("./".to_string());
    info!("Transcoder service started.");

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
                        if let Some(mut existing_process) = ffmpeg_processes.remove(&stream_id) {
                            info!(
                                "Existing transcoder found for stream \"{stream_id}\", terminating and restarting..."
                            );
                            existing_process.kill().await?;
                        }

                        let hls_output_dir = PathBuf::from(format!("{output_dir}/{stream_id}"));
                        if hls_output_dir.exists() {
                            remove_dir(&hls_output_dir).await?;
                        }
                        create_dir(&hls_output_dir).await?;

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
                            .arg(format!("{}/stream.m3m8", hls_output_dir.display()));

                        let child = command.spawn().context("Failed to spawn ffmpeg process")?;
                        ffmpeg_processes.insert(stream_id, child);
                    }
                    StreamEvent::Stop { stream_id } => {
                        info!("Stopping transcoder for stream: {stream_id}");
                        let Some(mut child) = ffmpeg_processes.remove(&stream_id) else {
                            warn!("transcoder has already been stopped.");
                            continue;
                        };
                        child
                            .kill()
                            .await
                            .context("Could not kill ffmpeg process")?;

                        let hls_output_dir = PathBuf::from(format!("{output_dir}/{stream_id}"));
                        if hls_output_dir.exists() {
                            remove_dir(&hls_output_dir).await?;
                        }

                        info!("Successfully stopped transcoding for stream: {stream_id}")
                    }
                };

                let _: () = conn.xack(STREAM_EVENT_KEY, TRANSCODER_GROUP, &[&entry.id])?;
            }
        }
    }
}
