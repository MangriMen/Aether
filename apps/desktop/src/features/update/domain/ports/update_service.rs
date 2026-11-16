use std::{future::Future, pin::Pin};

use crate::features::update::{UpdateError, UpdateStatus};
use async_trait::async_trait;

/// A check made after the update is downloaded and before it is installed, which restarts the
/// launcher. An error stops the install.
pub type BeforeInstall<'a> =
    &'a (dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), UpdateError>> + Send>> + Send + Sync);

#[async_trait]
pub trait UpdateService: Send + Sync {
    async fn check(&self) -> Result<Option<UpdateStatus>, UpdateError>;
    /// Downloads and installs the pending update, then restarts. `before_install` runs between
    /// the two: the download takes a while and what was true before it may not be after.
    async fn install(&self, before_install: BeforeInstall<'_>) -> Result<(), UpdateError>;
}
