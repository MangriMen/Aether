mod manage_process_service;
mod process_probe;
mod process_record_storage;
mod process_start_service;
mod track_process_service;

use async_trait::async_trait;
use process_wrap::tokio::ChildWrapper;
use uuid::Uuid;

use crate::features::process::domain::{MinecraftProcessMetadata, ProcessError, ProcessState};

pub use manage_process_service::ManageProcessService;
pub use process_probe::{AdoptedProcess, ProcessProbe};
pub use process_record_storage::ProcessRecordStorage;
pub use process_start_service::ProcessStartService;
pub use track_process_service::TrackProcessService;

/// The game process as the launcher holds it.
#[derive(Debug)]
pub enum ProcessHandle {
    /// A child of this launcher.
    Owned(Box<dyn ChildWrapper>),
    /// A game an earlier launcher started.
    Adopted(Box<dyn AdoptedProcess>),
}

#[async_trait]
pub trait ProcessStorage: Send + Sync {
    async fn insert(
        &self,
        process: MinecraftProcessMetadata,
        handle: ProcessHandle,
    ) -> Result<(), ProcessError>;
    async fn remove(&self, id: Uuid) -> Result<(), ProcessError>;

    async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
    async fn get_metadata(
        &self,
        id: Uuid,
    ) -> Result<Option<MinecraftProcessMetadata>, ProcessError>;

    /// `None` when the storage does not hold the process.
    async fn try_wait(&self, id: Uuid) -> Result<Option<ProcessState>, ProcessError>;
    /// Ask the process to stop and return without waiting for it: the exit is seen by `try_wait`.
    async fn kill(&self, id: Uuid) -> Result<(), ProcessError>;
    /// Whether `kill` was requested for the process (the exit is then not a crash).
    async fn was_killed(&self, id: Uuid) -> bool;
}

// ── Use case ports ──

#[async_trait]
pub trait KillProcessUseCasePort: Send + Sync {
    async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError>;
}

#[async_trait]
pub trait ListProcessMetadataUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
}
