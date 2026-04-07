use std::sync::Arc;

use crate::features::{
    java::{Java, JavaInstallationService, JavaStorage, JreProvider},
    settings::LocationInfo,
};

use super::super::JavaApplicationError;

pub struct InstallJavaUseCase<JS: JavaStorage, JIS: JavaInstallationService, JP: JreProvider> {
    storage: Arc<JS>,
    java_installation_service: JIS,
    provider: Arc<JP>,
    location_info: Arc<LocationInfo>,
}

impl<JS: JavaStorage, JIS: JavaInstallationService, JP: JreProvider>
    InstallJavaUseCase<JS, JIS, JP>
{
    pub fn new(
        storage: Arc<JS>,
        java_installation_service: JIS,
        provider: Arc<JP>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
            provider,
            location_info,
        }
    }

    pub async fn execute(&self, version: u32) -> Result<Java, JavaApplicationError> {
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
