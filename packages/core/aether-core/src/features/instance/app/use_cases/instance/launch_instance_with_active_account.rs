use std::sync::Arc;

use crate::features::{
    auth::{ActiveAccountHelper, CredentialsStorage},
    instance::InstanceError,
    process::MinecraftProcessMetadata,
};

use super::LaunchInstanceUseCase;

pub struct LaunchInstanceWithActiveAccountUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
    launch_instance_use_case: LaunchInstanceUseCase,
}

impl LaunchInstanceWithActiveAccountUseCase {
    pub fn new(
        credentials_storage: Arc<dyn CredentialsStorage>,
        launch_with_credentials_use_case: LaunchInstanceUseCase,
    ) -> Self {
        Self {
            credentials_storage,
            launch_instance_use_case: launch_with_credentials_use_case,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        let default_account =
            ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await?;

        self.launch_instance_use_case
            .execute(instance_id, default_account)
            .await
    }
}
