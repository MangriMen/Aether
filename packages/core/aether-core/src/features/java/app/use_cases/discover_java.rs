use std::{path::PathBuf, sync::Arc};

use crate::features::java::{Java, JavaInstallationService, app::JavaApplicationError};

pub struct DiscoverJavaUseCase<JIS: JavaInstallationService> {
    java_installation_service: Arc<JIS>,
    discovery_paths: Vec<PathBuf>,
}

impl<JIS: JavaInstallationService> DiscoverJavaUseCase<JIS> {
    pub fn new(java_installation_service: Arc<JIS>, discovery_paths: Vec<PathBuf>) -> Self {
        Self {
            java_installation_service,
            discovery_paths,
        }
    }

    pub async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError> {
        Ok(self
            .java_installation_service
            .discover_installations(&self.discovery_paths)
            .await?)
    }
}
