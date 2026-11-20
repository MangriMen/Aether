use std::{future::Future, pin::Pin, sync::Arc};

use aether_core::features::process::RestartService;

use crate::features::update::{UpdateError, UpdateService};

/// How long the launch gate stays closed after the install, for the launcher to restart.
const RESTART_GRACE: std::time::Duration = std::time::Duration::from_secs(60);

pub struct InstallUpdateUseCase<US: UpdateService> {
    update_service: Arc<US>,
    restart: Arc<dyn RestartService>,
}

impl<US: UpdateService> InstallUpdateUseCase<US> {
    pub fn new(update_service: Arc<US>, restart: Arc<dyn RestartService>) -> Self {
        Self {
            update_service,
            restart,
        }
    }

    /// Installs the update and restarts the launcher, unless a game is running: it would be left
    /// without the launcher, which tracks its time, runs its post-exit hook and detects a crash.
    /// With `keep_running_games` the games that carry on across the restart (the next launcher
    /// follows them) do not stop it. No game is launched from the check before the install to the
    /// restart. Checked before the download too, as it is long and may be called off.
    pub async fn execute(&self, keep_running_games: bool) -> Result<(), UpdateError> {
        drop(self.permit(keep_running_games).await?);

        let restart = self.restart.clone();
        let permit = Arc::new(std::sync::Mutex::new(None));
        let held = permit.clone();
        let before_install = move || {
            let restart = restart.clone();
            let held = held.clone();
            Box::pin(async move {
                let permit = restart
                    .permit(keep_running_games)
                    .await
                    .map_err(|_| UpdateError::GameRunning)?;
                *held
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(permit);
                Ok(())
            }) as Pin<Box<dyn Future<Output = Result<(), UpdateError>> + Send>>
        };

        let result = self.update_service.install(&before_install).await;
        let permit = permit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        match (result, permit) {
            // The launcher is going: the gate stays closed until it is gone. Not for good, as the
            // restart may be put off (a window being recreated) and the launcher go on.
            (Ok(()), Some(permit)) => {
                tokio::spawn(async move {
                    tokio::time::sleep(RESTART_GRACE).await;
                    drop(permit);
                });
                Ok(())
            }
            (result, _) => result,
        }
    }

    /// Whether the games that run now would carry on across the restart.
    pub async fn games_carry_on(&self) -> bool {
        self.restart.games_carry_on().await
    }

    async fn permit(
        &self,
        keep_running_games: bool,
    ) -> Result<aether_core::features::process::RestartPermit, UpdateError> {
        self.restart
            .permit(keep_running_games)
            .await
            .map_err(|_| UpdateError::GameRunning)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use aether_core::features::instance::InstanceId;
    use aether_core::features::process::{
        LaunchGate, PrepareRestartUseCase, ProcessError, RunningGameService,
    };
    use aether_core::shared::error::DatabaseError;
    use async_trait::async_trait;
    use uuid::Uuid;

    use super::*;
    use crate::features::update::{BeforeInstall, UpdateStatus};

    struct MockUpdateService {
        install_result: Result<(), UpdateError>,
        installs: AtomicUsize,
    }

    #[async_trait]
    impl UpdateService for MockUpdateService {
        async fn check(&self) -> Result<Option<UpdateStatus>, UpdateError> {
            Ok(None)
        }

        async fn install(&self, before_install: BeforeInstall<'_>) -> Result<(), UpdateError> {
            before_install().await?;
            self.installs.fetch_add(1, Ordering::SeqCst);
            self.install_result.clone()
        }
    }

    /// The games running on each call in turn (the last one repeats), `None` for games that
    /// cannot be told about.
    struct MockProcesses(Vec<Option<usize>>, AtomicUsize, bool);

    impl MockProcesses {
        fn restart(lists: Vec<Option<usize>>) -> Arc<dyn RestartService> {
            Self::carrying_on(lists, false)
        }

        /// `carry_on`: whether the running games carry on across a restart.
        fn carrying_on(lists: Vec<Option<usize>>, carry_on: bool) -> Arc<dyn RestartService> {
            Arc::new(PrepareRestartUseCase::new(
                Arc::new(LaunchGate::default()),
                Arc::new(Self(lists, AtomicUsize::new(0), carry_on)),
            ))
        }
    }

    #[async_trait]
    impl RunningGameService for MockProcesses {
        async fn running_game(
            &self,
            _instance_id: &InstanceId,
        ) -> Result<Option<Uuid>, ProcessError> {
            unimplemented!()
        }

        async fn any_running(&self) -> Result<bool, ProcessError> {
            let call = self.1.fetch_add(1, Ordering::SeqCst).min(self.0.len() - 1);

            self.0[call].map(|n| n > 0).ok_or_else(|| {
                ProcessError::SavedGamesUnreadable(DatabaseError::new("database is locked"))
            })
        }

        async fn running_games_carry_on(&self) -> Result<bool, ProcessError> {
            Ok(self.2)
        }
    }

    fn service(install_result: Result<(), UpdateError>) -> Arc<MockUpdateService> {
        Arc::new(MockUpdateService {
            install_result,
            installs: AtomicUsize::new(0),
        })
    }

    #[tokio::test]
    async fn install_failure() {
        let use_case = InstallUpdateUseCase::new(
            service(Err(UpdateError::NoPendingUpdate)),
            MockProcesses::restart(vec![Some(0)]),
        );
        let result = use_case.execute(false).await;
        assert!(matches!(result, Err(UpdateError::NoPendingUpdate)));
    }

    #[tokio::test]
    async fn should_install_when_no_game_is_running() {
        let service = service(Ok(()));
        let use_case =
            InstallUpdateUseCase::new(service.clone(), MockProcesses::restart(vec![Some(0)]));

        assert!(use_case.execute(false).await.is_ok());
        assert_eq!(service.installs.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_refuse_to_install_when_a_game_is_running() {
        let service = service(Ok(()));
        let use_case =
            InstallUpdateUseCase::new(service.clone(), MockProcesses::restart(vec![Some(1)]));

        assert!(matches!(
            use_case.execute(false).await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn should_refuse_to_install_when_the_running_games_are_unknown() {
        let service = service(Ok(()));
        let use_case =
            InstallUpdateUseCase::new(service.clone(), MockProcesses::restart(vec![None]));

        assert!(matches!(
            use_case.execute(false).await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn should_refuse_to_restart_when_a_game_started_during_the_download() {
        let service = service(Ok(()));
        let use_case = InstallUpdateUseCase::new(
            service.clone(),
            MockProcesses::restart(vec![Some(0), Some(1)]),
        );

        assert!(matches!(
            use_case.execute(false).await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn should_install_under_games_that_carry_on_when_asked_to_keep_them() {
        let service = service(Ok(()));
        let use_case = InstallUpdateUseCase::new(
            service.clone(),
            MockProcesses::carrying_on(vec![Some(1)], true),
        );

        assert!(use_case.execute(true).await.is_ok());
        assert_eq!(service.installs.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_refuse_games_that_carry_on_unless_asked_to_keep_them() {
        let service = service(Ok(()));
        let use_case = InstallUpdateUseCase::new(
            service.clone(),
            MockProcesses::carrying_on(vec![Some(1)], true),
        );

        assert!(matches!(
            use_case.execute(false).await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn should_refuse_to_keep_games_that_would_not_carry_on() {
        let service = service(Ok(()));
        let use_case = InstallUpdateUseCase::new(
            service.clone(),
            MockProcesses::carrying_on(vec![Some(1)], false),
        );

        assert!(matches!(
            use_case.execute(true).await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    fn gated(
        gate: &Arc<LaunchGate>,
        install_result: Result<(), UpdateError>,
    ) -> InstallUpdateUseCase<MockUpdateService> {
        let games = Arc::new(MockProcesses(vec![Some(0)], AtomicUsize::new(0), false));
        InstallUpdateUseCase::new(
            service(install_result),
            Arc::new(PrepareRestartUseCase::new(gate.clone(), games)),
        )
    }

    #[tokio::test]
    async fn should_keep_the_launch_gate_closed_after_the_install_that_restarts() {
        let gate = Arc::new(LaunchGate::default());

        assert!(gated(&gate, Ok(())).execute(false).await.is_ok());

        assert!(gate.try_restart().is_none());
    }

    #[tokio::test]
    async fn should_open_the_launch_gate_again_when_the_install_fails() {
        let gate = Arc::new(LaunchGate::default());

        assert!(
            gated(&gate, Err(UpdateError::NoPendingUpdate))
                .execute(false)
                .await
                .is_err()
        );

        assert!(gate.try_restart().is_some());
    }

    #[tokio::test]
    async fn should_not_install_while_a_game_is_being_launched() {
        let gate = Arc::new(LaunchGate::default());
        let _launch = gate.enter_launch().await;

        assert!(matches!(
            gated(&gate, Ok(())).execute(true).await,
            Err(UpdateError::GameRunning)
        ));
    }
}
