mod manage_process_service;
mod process_start_service;
mod track_process_service;

use std::process::ExitStatus;

use async_trait::async_trait;
use tokio::process::Child;
use uuid::Uuid;

use crate::features::process::domain::{MinecraftProcessMetadata, ProcessError};

pub use manage_process_service::ManageProcessService;
pub use process_start_service::ProcessStartService;
pub use track_process_service::TrackProcessService;

#[async_trait]
pub trait ProcessStorage: Send + Sync {
    async fn insert(
        &self,
        process: MinecraftProcessMetadata,
        child: Child,
    ) -> Result<(), ProcessError>;
    async fn remove(&self, id: Uuid) -> Result<(), ProcessError>;

    async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
    async fn get_metadata(
        &self,
        id: Uuid,
    ) -> Result<Option<MinecraftProcessMetadata>, ProcessError>;

    async fn try_wait(&self, id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError>;
    async fn wait_for(&self, id: Uuid) -> Result<(), ProcessError>;
    async fn kill(&self, id: Uuid) -> Result<(), ProcessError>;
}

// ── Use case ports ──

#[async_trait]
pub trait WaitForProcessUseCasePort: Send + Sync {
    async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError>;
}

#[async_trait]
pub trait KillProcessUseCasePort: Send + Sync {
    async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError>;
}

#[async_trait]
pub trait ListProcessMetadataUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
}

#[async_trait]
pub trait GetProcessMetadataByInstanceIdUseCasePort: Send + Sync {
    async fn execute(
        &self,
        instance_id: String,
    ) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
}
