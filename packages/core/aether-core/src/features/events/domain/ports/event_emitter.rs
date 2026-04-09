use std::sync::Arc;

use async_trait::async_trait;

use crate::features::events::EventError;

#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit_raw(&self, event: &str, payload: serde_json::Value) -> Result<(), EventError>;

    fn listen_raw(&self, event: String, handler: Box<dyn Fn(String) + Send + 'static>);
}

pub type SharedEventEmitter = Arc<dyn EventEmitter>;
