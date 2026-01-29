use std::env;

use anyhow::{Context, anyhow};
use castelia_events::stream_events::{STREAM_EVENT_KEY, StreamEvent};
use redis::{
    Commands,
    streams::{StreamReadOptions, StreamReadReply},
};
use sqlx::Pool;
use tracing::{error, info, instrument};

const BROADCAST_GROUP: &str = "broadcasts";

#[instrument(skip_all, name = "Broadcast event handler")]
pub async fn handle_events(pool: Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let client = redis::Client::open(env::var("REDIS_URL").context("REDIS_URL is missing")?)
        .context("Failed to initialize redis client")?;

    let mut conn = client.get_connection()?;

    let result: redis::RedisResult<()> =
        conn.xgroup_create_mkstream(STREAM_EVENT_KEY, BROADCAST_GROUP, "0");
    match result {
        Ok(_) => info!("Consumer group created successfully."),
        Err(e) if e.code() == Some("BUSYGROUP") => {
            info!("Consumer group already exists, skipping creation.");
        }
        Err(e) => return Err(anyhow!(e)),
    };

    info!("Initialized event handler");
    loop {
        let mut con = client.get_connection()?;
        let reply: StreamReadReply = tokio::task::spawn_blocking(move || {
            let opts = StreamReadOptions::default()
                .group(BROADCAST_GROUP, "broadcast-receiver")
                .block(5000);

            con.xread_options(&[STREAM_EVENT_KEY], &[">"], &opts)
        })
        .await??;

        for stream in reply.keys {
            for entry in stream.ids {
                let Some(payload) = entry.map.get("data") else {
                    error!("Missing payload on stream event");
                    continue;
                };
                let payload: String = redis::from_redis_value(payload.clone())?;
                let event: StreamEvent = serde_json::from_str(&payload)?;
                match event {
                    StreamEvent::Start { stream_id, .. } => {
                        sqlx::query!(
                            "UPDATE broadcasts SET status = CASE 
                                WHEN status = 'offline' THEN 'unpublished' 
                                ELSE status 
                            END 
                            WHERE channel_name = $1",
                            stream_id
                        )
                        .execute(&pool)
                        .await?;
                        info!(channel = %stream_id, "Broadcast is now LIVE");
                    }
                    StreamEvent::Stop { stream_id } => {
                        sqlx::query!(
                            "UPDATE broadcasts SET status = 'offline', start_time = NULL WHERE channel_name = $1",
                            stream_id
                        )
                        .execute(&pool)
                        .await?;
                        info!(channel = %stream_id, "Broadcast is now OFFLINE");
                    }
                };

                let _: () = conn.xack(STREAM_EVENT_KEY, BROADCAST_GROUP, &[&entry.id])?;
            }
        }
    }
}
