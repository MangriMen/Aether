use std::sync::Arc;

use async_trait::async_trait;

use super::super::super::domain::{MinecraftProcessMetadata, ProcessError};
use super::super::ports::{ListProcessMetadataUseCasePort, ProcessStorage};

pub struct ListProcessMetadataUseCase {
    process_storage: Arc<dyn ProcessStorage>,
}

impl ListProcessMetadataUseCase {
    pub fn new(process_storage: Arc<dyn ProcessStorage>) -> Self {
        Self { process_storage }
    }
}

#[async_trait]
impl ListProcessMetadataUseCasePort for ListProcessMetadataUseCase {
    async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
        self.process_storage.list_metadata().await
    }
}
