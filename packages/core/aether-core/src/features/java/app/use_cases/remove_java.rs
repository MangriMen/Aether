use std::sync::Arc;

use crate::features::java::{JavaStorage, app::JavaApplicationError};

pub struct RemoveJavaUseCase<JS: JavaStorage> {
    java_storage: Arc<JS>,
}

impl<JS: JavaStorage> RemoveJavaUseCase<JS> {
    pub fn new(java_storage: Arc<JS>) -> Self {
        Self { java_storage }
    }

    pub async fn execute(&self, major_version: u32) -> Result<(), JavaApplicationError> {
        self.java_storage.remove(major_version).await?;
        Ok(())
    }
}
