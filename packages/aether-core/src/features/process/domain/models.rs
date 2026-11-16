use chrono::{DateTime, Utc};

use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::shared::io::domain::IoError;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Failed to kill process {id}")]
    KillError { id: String },

    #[error("Failed to wait process {id}")]
    WaitError { id: String },

    #[error(transparent)]
    Io(#[from] IoError),
}

/// What stopping a started process ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillScope {
    /// The started process alone.
    Process,
    /// The started process and every process it starts: a job object on Windows, a process
    /// group on Unix.
    Tree,
}

#[derive(Debug, Clone)]
pub struct MinecraftProcessMetadata {
    uuid: Uuid,
    instance_id: InstanceId,
    start_time: DateTime<Utc>,
}

impl MinecraftProcessMetadata {
    pub fn new(instance_id: InstanceId) -> Self {
        MinecraftProcessMetadata {
            uuid: Uuid::new_v4(),
            instance_id,
            start_time: Utc::now(),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }
    pub fn start_time(&self) -> &DateTime<Utc> {
        &self.start_time
    }
}
