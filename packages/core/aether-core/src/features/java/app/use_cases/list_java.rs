use std::sync::Arc;

use crate::features::java::{Java, JavaStorage};

use super::super::JavaApplicationError;

pub struct ListJavaUseCase {
    storage: Arc<dyn JavaStorage>,
}

impl ListJavaUseCase {
    pub fn new(storage: Arc<dyn JavaStorage>) -> Self {
        Self { storage }
    }

    pub async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError> {
        Ok(self.storage.list().await?)
    }
}
