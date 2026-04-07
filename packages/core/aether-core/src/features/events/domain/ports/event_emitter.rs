use async_trait::async_trait;
use serde::Serialize;

use crate::features::events::EventError;

#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit<P: Serialize + Clone + Send>(
        &self,
        event: &str,
        payload: P,
    ) -> Result<(), EventError>;

    fn listen<F, T>(&self, event: impl Into<String>, handler: F)
    where
        F: Fn(String) + Send + 'static;
}
