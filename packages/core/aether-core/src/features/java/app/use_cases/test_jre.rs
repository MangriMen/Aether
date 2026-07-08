use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::features::java::{
    Java, JavaInstallationService,
    app::{JavaApplicationError, ports::TestJreUseCasePort},
};

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
        TestJreUseCasePort::execute(self, path).await
    }
}

#[async_trait]
impl TestJreUseCasePort for TestJreUseCase {
    async fn execute(&self, path: PathBuf) -> Result<Java, JavaApplicationError> {
        Ok(self.java_installation_service.locate_java(&path).await?)
    }
}
