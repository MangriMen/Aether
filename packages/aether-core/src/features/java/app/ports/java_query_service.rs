use async_trait::async_trait;

use crate::features::java::{Java, JavaApplicationError};

#[async_trait]
pub trait JavaQueryService: Send + Sync {
    async fn execute(&self, version: u32) -> Result<Java, JavaApplicationError>;
}
