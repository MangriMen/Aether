use std::sync::Arc;

use crate::features::{
    auth::{ActiveAccountHelper, CredentialsStorage},
    events::{EventEmitter, ProgressService},
    instance::{InstanceError, InstanceStorage},
    java::{JavaInstallationService, JavaStorage, JreProvider},
    minecraft::{MetadataStorage, MinecraftDownloader},
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
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
> {
    credentials_storage: Arc<CS>,
    launch_instance_use_case: LaunchInstanceUseCase<IS, MS, PS, GISS, E, MD, PGS, JIS, JS, JP>,
}

impl<
        IS: InstanceStorage + 'static,
        MS: MetadataStorage,
        PS: ProcessStorage + 'static,
        CS: CredentialsStorage,
        GISS: DefaultInstanceSettingsStorage,
        E: EventEmitter + 'static,
        MD: MinecraftDownloader,
        PGS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        JP: JreProvider,
    > LaunchInstanceWithActiveAccountUseCase<IS, MS, PS, CS, GISS, E, MD, PGS, JIS, JS, JP>
{
    pub fn new(
        credentials_storage: Arc<CS>,
        launch_with_credentials_use_case: LaunchInstanceUseCase<
            IS,
            MS,
            PS,
            GISS,
            E,
            MD,
            PGS,
            JIS,
            JS,
            JP,
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
