use std::{path::Path, sync::Arc};

use crate::features::java::{Java, JavaDomainError, JavaInstallationService, JavaStorage};

use super::super::JavaApplicationError;

pub struct GetJavaUseCase<JS: JavaStorage, JIS: JavaInstallationService> {
    storage: Arc<JS>,
    java_installation_service: JIS,
}

impl<JS: JavaStorage, JIS: JavaInstallationService> GetJavaUseCase<JS, JIS> {
    pub fn new(storage: Arc<JS>, java_installation_service: JIS) -> Self {
        Self {
            storage,
            java_installation_service,
        }
    }

    pub async fn execute(&self, version: u32) -> Result<Java, JavaApplicationError> {
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
