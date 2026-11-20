use std::fmt::Debug;

use async_trait::async_trait;

use crate::features::process::domain::{OsProcessIdentity, ProcessRecord, ProcessState};

/// What the OS tells about running processes.
#[async_trait]
pub trait ProcessProbe: Send + Sync {
    /// The identity of the running process `pid`; an error when the OS does not report it, for
    /// instance because the process is gone.
    async fn identity_of(&self, pid: u32) -> std::io::Result<OsProcessIdentity>;

    /// The identity of the launcher itself, read once; `None` when the OS did not report it.
    fn own_identity(&self) -> Option<&OsProcessIdentity>;

    /// Takes over the process of `record` so that it can be followed and stopped like a started
    /// one. `Ok(None)` only when the process is known to be gone: it has ended, or its pid is
    /// free or belongs to another process. An error when the OS could not tell: the game may
    /// still run, and its record must stay.
    async fn adopt(
        &self,
        record: &ProcessRecord,
    ) -> std::io::Result<Option<Box<dyn AdoptedProcess>>>;

    /// Whether the process of `record` still runs; an error when the OS could not tell, as
    /// with [`adopt`](Self::adopt).
    async fn is_alive(&self, record: &ProcessRecord) -> std::io::Result<bool>;
}

/// A running process that the launcher did not start. Both calls may block on the OS, so they
/// are made off the async runtime.
pub trait AdoptedProcess: Debug + Send + Sync {
    fn try_wait(&mut self) -> std::io::Result<ProcessState>;

    /// Asks the process to stop and returns without waiting for it: the exit is seen by
    /// `try_wait`.
    fn start_kill(&mut self) -> std::io::Result<()>;
}
