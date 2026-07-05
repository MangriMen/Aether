use std::sync::Arc;

use super::super::super::domain::{MinecraftProcessMetadata, ProcessError};
use super::super::ports::ProcessStorage;

pub struct ListProcessMetadataUseCase {
    process_storage: Arc<dyn ProcessStorage>,
}

impl ListProcessMetadataUseCase {
    pub fn new(process_storage: Arc<dyn ProcessStorage>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
        self.process_storage.list_metadata().await
    }
}
