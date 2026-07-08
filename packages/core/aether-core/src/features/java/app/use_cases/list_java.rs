use std::sync::Arc;

use async_trait::async_trait;

use crate::features::java::{
    Java, JavaStorage,
    app::{JavaApplicationError, ports::ListJavaUseCasePort},
};

pub struct ListJavaUseCase {
    storage: Arc<dyn JavaStorage>,
}

impl ListJavaUseCase {
    pub fn new(storage: Arc<dyn JavaStorage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl ListJavaUseCasePort for ListJavaUseCase {
    async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError> {
        Ok(self.storage.list().await?)
    }
}
