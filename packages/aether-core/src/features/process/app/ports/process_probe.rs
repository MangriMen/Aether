use async_trait::async_trait;

use crate::features::process::domain::OsProcessIdentity;

/// What the OS tells about running processes.
#[async_trait]
pub trait ProcessProbe: Send + Sync {
    /// The identity of the running process `pid`; an error when the OS does not report it, for
    /// instance because the process is gone.
    async fn identity_of(&self, pid: u32) -> std::io::Result<OsProcessIdentity>;

    /// The identity of the launcher itself, read once; `None` when the OS did not report it.
    fn own_identity(&self) -> Option<&OsProcessIdentity>;
}
