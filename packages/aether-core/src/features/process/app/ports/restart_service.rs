use async_trait::async_trait;

use crate::features::process::app::RestartPermit;

/// The launcher is not to restart now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("The launcher cannot restart now: a game would be lost")]
pub struct RestartRefused;

/// Whether the launcher may restart, as an update does, without losing a game it follows.
#[async_trait]
pub trait RestartService: Send + Sync {
    /// Closes the launch gate for a restart, held until the permit is dropped. Refused while a
    /// launch is going on. Refused while a game runs too, unless `keep_running_games` and every
    /// game carries on across the restart, with its record saved, so that the next launcher
    /// follows it as this one did. Games that cannot be told about count as running.
    async fn permit(&self, keep_running_games: bool) -> Result<RestartPermit, RestartRefused>;

    /// Whether the games that run now would carry on across a restart; true when none runs.
    async fn games_carry_on(&self) -> bool;
}
