use std::{path::PathBuf, sync::Arc};

use crate::features::{
    java::{
        Java, JavaInstallationService, JavaInstallationTracker, JavaStorage, JreProvider,
        app::InstallJava,
    },
    settings::LocationInfo,
};

use super::super::JavaApplicationError;

pub struct InstallJavaUseCase<
    JS: JavaStorage,
    JIS: JavaInstallationService,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> {
    storage: Arc<JS>,
    java_installation_service: JIS,
    provider: Arc<JP>,
    location_info: Arc<LocationInfo>,
    java_installation_tracker: Arc<JIT>,
}

impl<JS: JavaStorage, JIS: JavaInstallationService, JP: JreProvider, JIT: JavaInstallationTracker>
    InstallJavaUseCase<JS, JIS, JP, JIT>
{
    pub fn new(
        storage: Arc<JS>,
        java_installation_service: JIS,
        provider: Arc<JP>,
        location_info: Arc<LocationInfo>,
        java_installation_tracker: Arc<JIT>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
            provider,
            location_info,
            java_installation_tracker,
        }
    }

    pub async fn execute(&self, install_java: InstallJava) -> Result<Java, JavaApplicationError> {
        let InstallJava { version, force } = install_java;

        let _guard = self.java_installation_tracker.lock(version).await;

        if !force
            && let Ok(Some(existing_java)) = self.storage.get(version).await
            && self
                .java_installation_service
                .locate_java(&PathBuf::from(existing_java.path().to_owned()))
                .await
                .is_ok()
        {
            return Ok(existing_java);
        }

        let installed_jre_path = self
            .provider
            .install(version, &self.location_info.java_dir())
            .await?;

        let java = self
            .java_installation_service
            .locate_java(&installed_jre_path)
            .await?;

        Ok(self.storage.upsert(java).await?)
    }
}
