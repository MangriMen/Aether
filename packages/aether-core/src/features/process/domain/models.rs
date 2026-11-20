use std::process::ExitStatus;
use std::sync::Arc;

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

    /// The game was started by an earlier launcher, and this one has not taken it over yet:
    /// it cannot be stopped until then.
    #[error("The game process {id} has not been taken over yet")]
    NotTakenOverYet { id: String },

    /// The program of the command could not be started. Names the program only: the arguments
    /// of a launch hold the access token.
    #[error("Failed to start \"{program}\": {source}")]
    SpawnFailed {
        program: String,
        #[source]
        source: Arc<std::io::Error>,
    },

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

/// Whether a followed process still runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    /// The process ended. The status is unknown for a process the launcher did not start on
    /// Unix: only its parent may read it.
    Exited(Option<ExitStatus>),
}

/// How the launcher came to follow a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOrigin {
    /// This launcher started it.
    Launched,
    /// An earlier launcher started it, and this one took it over.
    Adopted,
}

#[derive(Debug, Clone)]
pub struct MinecraftProcessMetadata {
    uuid: Uuid,
    instance_id: InstanceId,
    start_time: DateTime<Utc>,
    origin: ProcessOrigin,
}

impl MinecraftProcessMetadata {
    pub fn new(instance_id: InstanceId) -> Self {
        MinecraftProcessMetadata {
            uuid: Uuid::new_v4(),
            instance_id,
            start_time: Utc::now(),
            origin: ProcessOrigin::Launched,
        }
    }

    /// A run that an earlier launcher started, under the id and start it was given then.
    pub fn adopted(uuid: Uuid, instance_id: InstanceId, start_time: DateTime<Utc>) -> Self {
        MinecraftProcessMetadata {
            uuid,
            instance_id,
            start_time,
            origin: ProcessOrigin::Adopted,
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
    pub fn origin(&self) -> ProcessOrigin {
        self.origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_the_id_and_start_of_the_run_when_it_is_adopted() {
        let launched = MinecraftProcessMetadata::new(InstanceId::generate());

        let adopted = MinecraftProcessMetadata::adopted(
            launched.uuid(),
            launched.instance_id().clone(),
            *launched.start_time(),
        );

        assert_eq!(launched.origin(), ProcessOrigin::Launched);
        assert_eq!(adopted.origin(), ProcessOrigin::Adopted);
        assert_eq!(adopted.uuid(), launched.uuid());
        assert_eq!(adopted.instance_id(), launched.instance_id());
        assert_eq!(adopted.start_time(), launched.start_time());
    }
}
