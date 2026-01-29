use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use castelia_events::stream_events::{STREAM_EVENT_KEY, StreamEvent};
use redis::{
    Client, Commands, Connection,
    streams::{StreamReadOptions, StreamReadReply},
};
use tokio::{
    fs::{create_dir, remove_dir_all},
    process::{Child, Command},
    signal,
};
use tracing::{error, info, instrument, trace, warn};

use crate::TRANSCODER_GROUP;

pub struct TranscoderService {
    processes: HashMap<String, Child>,
    output_dir: PathBuf,
    client: Client,
}

impl TranscoderService {
    pub fn new(output_dir: &Path, client: Client) -> Self {
        Self {
            processes: HashMap::new(),
            output_dir: output_dir.to_path_buf(),
            client,
        }
    }

    #[instrument(skip_all, fields(stream_id = %stream_id))]
    async fn stop_transcoder(&mut self, stream_id: &str) -> Result<()> {
        let Some(mut child) = self.processes.remove(stream_id) else {
            warn!("Transcoder for \"{stream_id}\" has already been stopped.");
            return Ok(());
        };

        child
            .kill()
            .await
            .context("Could not kill ffmpeg process")?;

        let hls_output_dir = PathBuf::from(format!("{}/{stream_id}", self.output_dir.display()));
        if hls_output_dir.exists() {
            remove_dir_all(&hls_output_dir).await?;
        }

        info!("Transcoder process stopped");

        Ok(())
    }

    #[instrument(skip_all, fields(stream_id = %stream_id))]
    async fn spawn_transcoder(&mut self, stream_id: &str, rtmp_url: &str) -> Result<()> {
        if let Some(mut existing_process) = self.processes.remove(stream_id) {
            info!(
                "Existing transcoder found for stream \"{stream_id}\", terminating and restarting..."
            );
            existing_process.kill().await?;
        }

        let hls_output_dir = PathBuf::from(format!("{}/{stream_id}", self.output_dir.display()));
        if hls_output_dir.exists() {
            remove_dir_all(&hls_output_dir).await?;
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
            .arg(format!("{}/stream.m3u8", hls_output_dir.display()));

        let child = command.spawn().context("Failed to spawn ffmpeg process")?;
        self.processes.insert(stream_id.to_owned(), child);
        info!("Transcoder process spawned");
        Ok(())
    }

    async fn process_events(
        &mut self,
        conn: &mut Connection,
        opts: &StreamReadOptions,
    ) -> anyhow::Result<()> {
        let reply: StreamReadReply = conn.xread_options(&[STREAM_EVENT_KEY], &[">"], opts)?;

        for stream in reply.keys {
            for entry in stream.ids {
                trace!("{:?}", &entry);
                let Some(payload) = entry.map.get("data") else {
                    error!("Missing payload on stream event");
                    continue;
                };
                let payload: String = redis::from_redis_value(payload.clone())?;
                let event: StreamEvent = serde_json::from_str(&payload)?;
                match event {
                    StreamEvent::Start {
                        stream_id,
                        rtmp_url,
                    } => {
                        self.spawn_transcoder(&stream_id, &rtmp_url).await?;
                    }
                    StreamEvent::Stop { stream_id } => {
                        self.stop_transcoder(&stream_id).await?;
                    }
                };

                let _: () = conn.xack(STREAM_EVENT_KEY, TRANSCODER_GROUP, &[&entry.id])?;
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn clean_up(&mut self) -> anyhow::Result<()> {
        for process in self.processes.values_mut() {
            process
                .kill()
                .await
                .context("Failed to terminate transcoder process")?
        }
        info!("Terminated {} processes", self.processes.len());

        for hls_dir in self.output_dir.read_dir()? {
            remove_dir_all(hls_dir?.path())
                .await
                .context("Failed to remove HLS dir")?;
        }
        info!("Cleaned up output dir {}", self.output_dir.display());

        Ok(())
    }

    #[instrument(skip_all, name = "transcoder_service")]
    pub async fn start(&mut self) -> Result<()> {
        info!("Transcoder service started.");
        let mut conn = self.client.get_connection()?;
        let opts = StreamReadOptions::default()
            .group(TRANSCODER_GROUP, "transcoder")
            .block(5000);

        #[allow(clippy::expect_used)]
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler");

        loop {
            tokio::select! {
                result = self.process_events(&mut conn, &opts) => {
                    if let Err(e) = result {
                        error!("Failed to process event: {e}");
                    }
                }
                _ = sigterm.recv() => {
                    info!("Shutdown signal received. Cleaning up...");
                    self.clean_up().await.context("Failure while cleaning up")?;
                    break;

                }
            };
        }
        Ok(())
    }
}
