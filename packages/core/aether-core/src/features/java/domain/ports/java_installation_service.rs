use std::path::Path;

use async_trait::async_trait;

use crate::features::java::{Java, JavaDomainError};

#[async_trait]
pub trait JavaInstallationService: Send + Sync {
    async fn locate_java(&self, path: &Path) -> Result<Java, JavaDomainError>;
}
