use async_trait::async_trait;

use crate::{features::java::Java, shared::IoError};

#[async_trait]
pub trait JavaStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Java>, JavaStorageError>;
    async fn get(&self, version: u32) -> Result<Option<Java>, JavaStorageError>;
    async fn upsert(&self, java: Java) -> Result<Java, JavaStorageError>;
}

#[derive(Debug, thiserror::Error)]
pub enum JavaStorageError {
    #[error("IO error: {0}")]
    Io(#[from] IoError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
