use std::sync::Arc;

use super::super::super::domain::{MinecraftProcessMetadata, ProcessError};
use super::super::ports::ProcessStorage;

pub struct ListProcessMetadataUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> ListProcessMetadataUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
        self.process_storage.list_metadata().await
    }
}
