use std::{path::PathBuf, sync::Arc};

use crate::features::java::JavaInstallationService;

pub struct TestJreVersion {
    pub major_version: u32,
    pub path: PathBuf,
}

pub struct TestJreUseCase<JIS: JavaInstallationService> {
    java_installation_service: Arc<JIS>,
}

impl<JIS: JavaInstallationService> TestJreUseCase<JIS> {
    pub fn new(java_installation_service: Arc<JIS>) -> Self {
        Self {
            java_installation_service,
        }
    }

    pub async fn execute(&self, test_version: TestJreVersion) -> bool {
        let Ok(java) = self
            .java_installation_service
            .locate_java(&test_version.path)
            .await
        else {
            return false;
        };

        java.major_version() == test_version.major_version
    }
}
