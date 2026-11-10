use async_trait::async_trait;
use uuid::Uuid;

use crate::features::events::EventError;

/// Port trait for the cancel progress use case.
#[async_trait]
pub trait CancelProgressUseCasePort: Send + Sync {
    /// Call off the work of the bar `progress_bar_id`, as the user asked. A bar that is gone or
    /// cannot be called off is not an error: the work may have ended in the meantime.
    async fn execute(&self, progress_bar_id: Uuid) -> Result<(), EventError>;
}
