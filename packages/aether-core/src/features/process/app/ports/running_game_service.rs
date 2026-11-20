use async_trait::async_trait;
use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::features::process::domain::ProcessError;

/// Whether a game runs, as the launcher follows it and as the saved records tell. A game the OS
/// cannot tell about, or one another running launcher follows, counts as running; records that
/// cannot be read are an error, which callers take as a game that runs.
#[async_trait]
pub trait RunningGameService: Send + Sync {
    /// The id of the run of the instance's game that is going on. A record whose game is gone
    /// is forgotten on the way.
    async fn running_game(&self, instance_id: &InstanceId) -> Result<Option<Uuid>, ProcessError>;

    /// Whether the game of any instance runs. Changes nothing.
    async fn any_running(&self) -> Result<bool, ProcessError>;
}
