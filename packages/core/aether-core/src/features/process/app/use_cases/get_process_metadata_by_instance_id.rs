use std::sync::Arc;

use async_trait::async_trait;

use super::super::super::domain::{MinecraftProcessMetadata, ProcessError};
use super::super::ports::{GetProcessMetadataByInstanceIdUseCasePort, ProcessStorage};

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
        GetProcessMetadataByInstanceIdUseCasePort::execute(self, instance_id).await
    }
}

#[async_trait]
impl GetProcessMetadataByInstanceIdUseCasePort for GetProcessMetadataByInstanceIdUseCase {
    async fn execute(
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
