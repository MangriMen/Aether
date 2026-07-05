use std::sync::Arc;

use super::super::super::domain::{MinecraftProcessMetadata, ProcessError};
use super::super::ports::ProcessStorage;

pub struct GetProcessMetadataByInstanceIdUseCase {
    process_storage: Arc<dyn ProcessStorage>,
}

impl GetProcessMetadataByInstanceIdUseCase {
    pub fn new(process_storage: Arc<dyn ProcessStorage>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(
        &self,
        instance_id: String,
    ) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
        Ok(self
            .process_storage
            .list_metadata()
            .await?
            .into_iter()
            .filter(|x| x.instance_id() == instance_id)
            .collect())
    }
}
