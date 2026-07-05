use std::{path::PathBuf, sync::Arc};

use crate::features::java::{Java, JavaInstallationService, app::JavaApplicationError};

pub struct TestJreUseCase {
    java_installation_service: Arc<dyn JavaInstallationService>,
}

impl TestJreUseCase {
    pub fn new(java_installation_service: Arc<dyn JavaInstallationService>) -> Self {
        Self {
            java_installation_service,
        }
    }

    pub async fn execute(&self, path: PathBuf) -> Result<Java, JavaApplicationError> {
        Ok(self.java_installation_service.locate_java(&path).await?)
    }
}
