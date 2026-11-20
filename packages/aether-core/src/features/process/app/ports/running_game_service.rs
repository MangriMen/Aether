use async_trait::async_trait;
use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::features::process::domain::ProcessError;

#[async_trait]
pub trait RunningGameService: Send + Sync {
    /// The id of the run of the instance's game that is going on: one the launcher follows, or
    /// one only a saved record tells of. A record whose game is gone is forgotten on the way;
    /// one whose game the OS cannot tell about counts as running.
    async fn running_game(&self, instance_id: &InstanceId) -> Result<Option<Uuid>, ProcessError>;
}
