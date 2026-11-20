use std::sync::Arc;

use async_trait::async_trait;

use crate::features::process::{
    LaunchGate, RestartPermit, RestartRefused, RestartService, RunningGameService,
};

pub struct PrepareRestartUseCase {
    gate: Arc<LaunchGate>,
    games: Arc<dyn RunningGameService>,
}

impl PrepareRestartUseCase {
    pub fn new(gate: Arc<LaunchGate>, games: Arc<dyn RunningGameService>) -> Self {
        Self { gate, games }
    }

    pub async fn execute(&self, keep_running_games: bool) -> Result<RestartPermit, RestartRefused> {
        // The gate first: past it no game is launched, so the games looked at are the last ones.
        let permit = self.gate.try_restart().ok_or(RestartRefused)?;
        match self.games.any_running().await {
            Ok(false) => Ok(permit),
            Ok(true) if keep_running_games && self.games_carry_on().await => Ok(permit),
            Ok(true) | Err(_) => Err(RestartRefused),
        }
    }

    pub async fn games_carry_on(&self) -> bool {
        self.games.running_games_carry_on().await.unwrap_or(false)
    }
}

#[async_trait]
impl RestartService for PrepareRestartUseCase {
    async fn permit(&self, keep_running_games: bool) -> Result<RestartPermit, RestartRefused> {
        self.execute(keep_running_games).await
    }

    async fn games_carry_on(&self) -> bool {
        self.games_carry_on().await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::features::{instance::InstanceId, process::ProcessError};
    use crate::shared::error::DatabaseError;

    /// What the games are like: running or not, carrying on or not; `None` when unreadable.
    struct Games {
        running: Option<bool>,
        carry_on: Option<bool>,
    }

    #[async_trait]
    impl RunningGameService for Games {
        async fn running_game(&self, _: &InstanceId) -> Result<Option<Uuid>, ProcessError> {
            unimplemented!()
        }

        async fn any_running(&self) -> Result<bool, ProcessError> {
            self.running.ok_or_else(unreadable)
        }

        async fn running_games_carry_on(&self) -> Result<bool, ProcessError> {
            self.carry_on.ok_or_else(unreadable)
        }
    }

    fn unreadable() -> ProcessError {
        ProcessError::SavedGamesUnreadable(DatabaseError::new("database is locked"))
    }

    fn use_case(running: Option<bool>, carry_on: Option<bool>) -> PrepareRestartUseCase {
        PrepareRestartUseCase::new(
            Arc::new(LaunchGate::default()),
            Arc::new(Games { running, carry_on }),
        )
    }

    #[tokio::test]
    async fn should_permit_a_restart_when_no_game_runs() {
        assert!(use_case(Some(false), None).execute(false).await.is_ok());
    }

    #[tokio::test]
    async fn should_refuse_a_restart_under_a_game_unless_asked_to_keep_it() {
        let use_case = use_case(Some(true), Some(true));

        assert_eq!(use_case.execute(false).await.err(), Some(RestartRefused));
        assert!(use_case.execute(true).await.is_ok());
    }

    #[tokio::test]
    async fn should_refuse_a_restart_under_a_game_that_would_not_carry_on() {
        let use_case = use_case(Some(true), Some(false));

        assert_eq!(use_case.execute(true).await.err(), Some(RestartRefused));
    }

    #[tokio::test]
    async fn should_refuse_a_restart_when_the_games_cannot_be_told_about() {
        assert!(use_case(None, None).execute(true).await.is_err());
        assert!(use_case(Some(true), None).execute(true).await.is_err());
    }

    #[tokio::test]
    async fn should_refuse_a_restart_while_a_launch_is_going_on() {
        let use_case = use_case(Some(false), None);
        let launch = use_case.gate.enter_launch().await;

        assert_eq!(use_case.execute(true).await.err(), Some(RestartRefused));

        drop(launch);
        assert!(use_case.execute(true).await.is_ok());
    }

    #[tokio::test]
    async fn should_hold_the_gate_until_the_permit_is_dropped() {
        let use_case = use_case(Some(false), None);
        let permit = use_case.execute(false).await.unwrap();

        assert!(use_case.gate.try_restart().is_none());

        drop(permit);
        assert!(use_case.gate.try_restart().is_some());
    }
}
