use std::{path::PathBuf, sync::Arc};

use crate::features::java::{Java, JavaInstallationService, app::JavaApplicationError};

pub struct TestJreUseCase<JIS: JavaInstallationService> {
    java_installation_service: Arc<JIS>,
}

impl<JIS: JavaInstallationService> TestJreUseCase<JIS> {
    pub fn new(java_installation_service: Arc<JIS>) -> Self {
        Self {
            java_installation_service,
        }
    }

    pub async fn execute(&self, path: PathBuf) -> Result<Java, JavaApplicationError> {
        Ok(self.java_installation_service.locate_java(&path).await?)
    }
}
