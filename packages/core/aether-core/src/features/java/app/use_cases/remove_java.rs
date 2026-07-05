use std::sync::Arc;

use crate::features::java::{JavaStorage, app::JavaApplicationError};

pub struct RemoveJavaUseCase {
    java_storage: Arc<dyn JavaStorage>,
}

impl RemoveJavaUseCase {
    pub fn new(java_storage: Arc<dyn JavaStorage>) -> Self {
        Self { java_storage }
    }

    pub async fn execute(&self, major_version: u32) -> Result<(), JavaApplicationError> {
        self.java_storage.remove(major_version).await?;
        Ok(())
    }
}
