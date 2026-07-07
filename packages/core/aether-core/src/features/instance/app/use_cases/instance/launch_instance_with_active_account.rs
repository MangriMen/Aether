use std::sync::Arc;

use crate::features::{
    auth::{ActiveAccountHelper, CredentialsStorage},
    instance::{InstanceError, InstanceLaunchService},
    process::MinecraftProcessMetadata,
};

pub struct LaunchInstanceWithActiveAccountUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
    launch_instance: Arc<dyn InstanceLaunchService>,
}

impl LaunchInstanceWithActiveAccountUseCase {
    pub fn new(
        credentials_storage: Arc<dyn CredentialsStorage>,
        launch_instance: Arc<dyn InstanceLaunchService>,
    ) -> Self {
        Self {
            credentials_storage,
            launch_instance,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        let default_account =
            ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await?;

        self.launch_instance
            .execute(instance_id, default_account)
            .await
    }
}
