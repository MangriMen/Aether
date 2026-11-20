use std::process::ExitStatus;

use async_trait::async_trait;

use crate::features::process::app::use_cases::TrackProcessParams;

/// How the following of a game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedEnd {
    /// The game exited. The status is unknown for a game taken over from an earlier launcher on
    /// a platform that tells only its parent how it ended.
    Exited(Option<ExitStatus>),
    /// The launcher lost sight of the game: the OS refused to tell, or the game was not held.
    /// It may still run, so its record stays.
    Lost,
}

#[async_trait]
pub trait TrackProcessService: Send + Sync {
    async fn execute(&self, params: TrackProcessParams) -> TrackedEnd;
}
