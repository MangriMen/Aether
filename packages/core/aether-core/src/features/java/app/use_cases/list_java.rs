use std::sync::Arc;

use crate::features::java::{Java, JavaStorage};

use super::super::JavaApplicationError;

pub struct ListJavaUseCase<JS: JavaStorage> {
    storage: Arc<JS>,
}

impl<JS: JavaStorage> ListJavaUseCase<JS> {
    pub fn new(storage: Arc<JS>) -> Self {
        Self { storage }
    }

    pub async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError> {
        Ok(self.storage.list().await?)
    }
}
