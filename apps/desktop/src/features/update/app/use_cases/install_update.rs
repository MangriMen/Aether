use std::{future::Future, pin::Pin, sync::Arc};

use aether_core::features::process::ListProcessMetadataUseCasePort;

use crate::features::update::{UpdateError, UpdateService};

pub struct InstallUpdateUseCase<US: UpdateService> {
    update_service: Arc<US>,
    processes: Arc<dyn ListProcessMetadataUseCasePort>,
}

impl<US: UpdateService> InstallUpdateUseCase<US> {
    pub fn new(
        update_service: Arc<US>,
        processes: Arc<dyn ListProcessMetadataUseCasePort>,
    ) -> Self {
        Self {
            update_service,
            processes,
        }
    }

    /// Installs the update and restarts the launcher, unless a game is running: it would be left
    /// without the launcher, which tracks its time, runs its post-exit hook and detects a crash.
    /// Checked again after the download, as a game may have been started during it.
    pub async fn execute(&self) -> Result<(), UpdateError> {
        ensure_no_game_runs(self.processes.as_ref()).await?;

        let processes = self.processes.clone();
        let before_install = move || {
            let processes = processes.clone();
            Box::pin(async move { ensure_no_game_runs(processes.as_ref()).await })
                as Pin<Box<dyn Future<Output = Result<(), UpdateError>> + Send>>
        };

        self.update_service.install(&before_install).await
    }
}

async fn ensure_no_game_runs(
    processes: &dyn ListProcessMetadataUseCasePort,
) -> Result<(), UpdateError> {
    // A process that cannot be listed is not known to be absent.
    match processes.execute().await {
        Ok(running) if running.is_empty() => Ok(()),
        _ => Err(UpdateError::GameRunning),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use aether_core::features::instance::InstanceId;
    use aether_core::features::process::{MinecraftProcessMetadata, ProcessError};
    use async_trait::async_trait;

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

    /// The processes listed on each call in turn (the last one repeats), `None` for a list that
    /// cannot be read.
    struct MockProcesses(Vec<Option<usize>>, AtomicUsize);

    impl MockProcesses {
        fn new(lists: Vec<Option<usize>>) -> Self {
            Self(lists, AtomicUsize::new(0))
        }
    }

    #[async_trait]
    impl ListProcessMetadataUseCasePort for MockProcesses {
        async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
            let call = self.1.fetch_add(1, Ordering::SeqCst).min(self.0.len() - 1);

            self.0[call]
                .map(|n| {
                    (0..n)
                        .map(|_| MinecraftProcessMetadata::new(InstanceId::generate()))
                        .collect()
                })
                .ok_or(ProcessError::KillError { id: "x".to_owned() })
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
            Arc::new(MockProcesses::new(vec![Some(0)])),
        );
        let result = use_case.execute().await;
        assert!(matches!(result, Err(UpdateError::NoPendingUpdate)));
    }

    #[tokio::test]
    async fn should_install_when_no_game_is_running() {
        let service = service(Ok(()));
        let use_case =
            InstallUpdateUseCase::new(service.clone(), Arc::new(MockProcesses::new(vec![Some(0)])));

        assert!(use_case.execute().await.is_ok());
        assert_eq!(service.installs.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_refuse_to_install_when_a_game_is_running() {
        let service = service(Ok(()));
        let use_case =
            InstallUpdateUseCase::new(service.clone(), Arc::new(MockProcesses::new(vec![Some(1)])));

        assert!(matches!(
            use_case.execute().await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn should_refuse_to_install_when_the_running_games_are_unknown() {
        let service = service(Ok(()));
        let use_case =
            InstallUpdateUseCase::new(service.clone(), Arc::new(MockProcesses::new(vec![None])));

        assert!(matches!(
            use_case.execute().await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn should_refuse_to_restart_when_a_game_started_during_the_download() {
        let service = service(Ok(()));
        let use_case = InstallUpdateUseCase::new(
            service.clone(),
            Arc::new(MockProcesses::new(vec![Some(0), Some(1)])),
        );

        assert!(matches!(
            use_case.execute().await,
            Err(UpdateError::GameRunning)
        ));
        assert_eq!(service.installs.load(Ordering::SeqCst), 0);
    }
}
