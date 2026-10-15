use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::features::java::{
    Java, JavaInstallationService,
    app::{JavaApplicationError, ports::DiscoverJavaUseCasePort},
};

pub struct DiscoverJavaUseCase {
    java_installation_service: Arc<dyn JavaInstallationService>,
    discovery_paths: Vec<PathBuf>,
}

impl DiscoverJavaUseCase {
    pub fn new(
        java_installation_service: Arc<dyn JavaInstallationService>,
        discovery_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            java_installation_service,
            discovery_paths,
        }
    }
}

#[async_trait]
impl DiscoverJavaUseCasePort for DiscoverJavaUseCase {
    async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError> {
        Ok(self
            .java_installation_service
            .discover_installations(&self.discovery_paths)
            .await?)
    }
}
