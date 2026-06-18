use std::sync::Arc;

use crate::features::{
    auth::{ActiveAccountHelper, CredentialsStorage},
    events::ProgressService,
    instance::{InstanceError, InstanceStorage},
    java::{JavaInstallationService, JavaInstallationTracker, JavaStorage, JreProvider},
    minecraft::{MetadataStorage, MinecraftDownloader, ModLoaderProcessor},
    process::{MinecraftProcessMetadata, ProcessStorage},
    settings::DefaultInstanceSettingsStorage,
};

use super::LaunchInstanceUseCase;

pub struct LaunchInstanceWithActiveAccountUseCase<
    IS: InstanceStorage,
    MS: MetadataStorage,
    PS: ProcessStorage,
    CS: CredentialsStorage,
    GISS: DefaultInstanceSettingsStorage,
    MD: MinecraftDownloader,
    MLP: ModLoaderProcessor,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> {
    credentials_storage: Arc<CS>,
    launch_instance_use_case:
        LaunchInstanceUseCase<IS, MS, PS, GISS, MD, MLP, PGS, JIS, JS, JP, JIT>,
}

impl<
    IS: InstanceStorage + 'static,
    MS: MetadataStorage,
    PS: ProcessStorage + 'static,
    CS: CredentialsStorage,
    GISS: DefaultInstanceSettingsStorage,
    MD: MinecraftDownloader,
    MLP: ModLoaderProcessor,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> LaunchInstanceWithActiveAccountUseCase<IS, MS, PS, CS, GISS, MD, MLP, PGS, JIS, JS, JP, JIT>
{
    pub fn new(
        credentials_storage: Arc<CS>,
        launch_with_credentials_use_case: LaunchInstanceUseCase<
            IS,
            MS,
            PS,
            GISS,
            MD,
            MLP,
            PGS,
            JIS,
            JS,
            JP,
            JIT,
        >,
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
