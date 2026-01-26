use async_trait::async_trait;

#[async_trait]
pub trait StreamEventEmitter: Send + Sync {
    async fn on_published(&self);
    async fn on_close_stream(&self, stream_id: &str);
}
