use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::features::java::domain::JavaDomainError;

#[async_trait]
pub trait JreProvider: Send + Sync {
    /// Download and unpack the JRE into `install_dir`. With `cancellable` the user can call it
    /// off from the bar of the download until the installed JRE starts being replaced; the call
    /// then ends in [`JavaDomainError::Cancelled`] with the installed JRE as it was.
    async fn install(
        &self,
        version: u32,
        install_dir: &Path,
        cancellable: bool,
    ) -> Result<PathBuf, JavaDomainError>;
}
