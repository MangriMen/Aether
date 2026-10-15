use std::{path::Path, sync::Arc};

use crate::features::java::{
    Java, JavaApplicationError, JavaInstallationService, JavaQueryService, JavaStorage,
    domain::JavaDomainError,
};

pub struct GetJavaUseCase {
    storage: Arc<dyn JavaStorage>,
    java_installation_service: Arc<dyn JavaInstallationService>,
}

#[async_trait::async_trait]
impl JavaQueryService for GetJavaUseCase {
    async fn execute(&self, version: u32) -> Result<Java, JavaApplicationError> {
        let java = self
            .storage
            .get(version)
            .await?
            .ok_or(JavaDomainError::NotFound { version })?;

        Ok(self
            .java_installation_service
            .locate_java(Path::new(java.path()))
            .await?)
    }
}

impl GetJavaUseCase {
    pub fn new(
        storage: Arc<dyn JavaStorage>,
        java_installation_service: Arc<dyn JavaInstallationService>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
        }
    }
}
