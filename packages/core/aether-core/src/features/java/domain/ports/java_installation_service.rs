use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::features::java::{Java, JavaDomainError};

#[async_trait]
pub trait JavaInstallationService: Send + Sync {
    async fn locate_java(&self, path: &Path) -> Result<Java, JavaDomainError>;
    async fn discover_installations(
        &self,
        base_paths: &[PathBuf],
    ) -> Result<Vec<Java>, JavaDomainError>;
}
