use async_trait::async_trait;

use crate::features::events::ProgressBar;

/// Port trait for the list progress bars use case.
#[async_trait]
pub trait ListProgressBarsUseCasePort: Send + Sync {
    async fn execute(&self) -> Vec<ProgressBar>;
}
