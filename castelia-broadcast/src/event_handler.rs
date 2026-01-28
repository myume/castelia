use std::env;

use anyhow::{Context, anyhow};
use castelia_events::stream_events::{STREAM_EVENT_KEY, StreamEvent};
use redis::{
    AsyncCommands,
    aio::ConnectionManager,
    streams::{StreamReadOptions, StreamReadReply},
};
use sqlx::Pool;
use tracing::{debug, error, info, instrument};

const BROADCAST_GROUP: &str = "broadcasts";

#[instrument(skip_all, name = "Broadcast event handler")]
pub async fn handle_events(pool: Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let client = redis::Client::open(env::var("REDIS_URL").context("REDIS_URL is missing")?)
        .context("Failed to initialize redis client")?;

    let mut conn = ConnectionManager::new(client).await?;

    let result: redis::RedisResult<()> = conn
        .xgroup_create_mkstream(STREAM_EVENT_KEY, BROADCAST_GROUP, "0")
        .await;
    match result {
        Ok(_) => info!("Consumer group created successfully."),
        Err(e) if e.code() == Some("BUSYGROUP") => {
            info!("Consumer group already exists, skipping creation.");
        }
        Err(e) => return Err(anyhow!(e)),
    };
    let opts = StreamReadOptions::default()
        .group(BROADCAST_GROUP, "broadcast-receiver")
        .block(5000);

    loop {
        let reply = conn.xread_options(&[STREAM_EVENT_KEY], &[">"], &opts).await;

        let reply: StreamReadReply = match reply {
            Ok(r) => r,
            Err(e) => {
                if e.is_timeout() {
                    continue;
                }

                // 3. For real errors (like Redis being down), log it and wait before retrying
                error!("Redis connection error: {e}. Retrying in 2s...");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

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
                        debug!("broadcast is live for \"{stream_id}\"");
                        sqlx::query!(
                            "UPDATE broadcasts SET status = CASE 
                                WHEN status = 'offline' THEN 'live' 
                                ELSE status 
                            END 
                            WHERE channel_name = $1",
                            stream_id
                        )
                        .execute(&pool)
                        .await?
                    }
                    StreamEvent::Stop { stream_id } => {
                        debug!("broadcast is now offline for \"{stream_id}\"");
                        sqlx::query!(
                            "UPDATE broadcasts SET status = 'offline', start_time = NULL WHERE channel_name = $1",
                            stream_id
                        )
                        .execute(&pool)
                        .await?
                    }
                };

                let _: () = conn
                    .xack(STREAM_EVENT_KEY, BROADCAST_GROUP, &[&entry.id])
                    .await?;
            }
        }
    }
}
