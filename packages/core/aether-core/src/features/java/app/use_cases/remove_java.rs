use std::sync::Arc;

use async_trait::async_trait;

use crate::features::java::{
    JavaStorage,
    app::{JavaApplicationError, ports::RemoveJavaUseCasePort},
};

pub struct RemoveJavaUseCase {
    java_storage: Arc<dyn JavaStorage>,
}

impl RemoveJavaUseCase {
    pub fn new(java_storage: Arc<dyn JavaStorage>) -> Self {
        Self { java_storage }
    }

    pub async fn execute(&self, major_version: u32) -> Result<(), JavaApplicationError> {
        RemoveJavaUseCasePort::execute(self, major_version).await
    }
}

#[async_trait]
impl RemoveJavaUseCasePort for RemoveJavaUseCase {
    async fn execute(&self, major_version: u32) -> Result<(), JavaApplicationError> {
        self.java_storage.remove(major_version).await?;
        Ok(())
    }
}
