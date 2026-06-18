use std::sync::Arc;

use async_trait::async_trait;

use crate::features::events::{Event, EventError};

#[async_trait]
pub trait EventEmitter<E>: Send + Sync {
    async fn emit(&self, event: E) -> Result<(), EventError>;
    fn listen(&self, handler: Box<dyn Fn(E) + Send + Sync + 'static>);
}

pub type SharedEventEmitter<E = Event> = Arc<dyn EventEmitter<E>>;
