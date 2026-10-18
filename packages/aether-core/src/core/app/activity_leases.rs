//! Downloads and installs that hold the [`ActivityGate`] busy while they run.
//!
//! They take a lease so the user cannot empty the caches under them and the storage tab shows
//! busy. Nothing may take a lease while holding a pause (cache clearing takes none): a lease
//! waits for the pause to end.

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        events::ProgressBarId,
        instance::{ContentInstallParams, InstallContentUseCasePort, InstanceError},
        java::{InstallJava, Java, JavaApplicationError, JavaInstallService},
        minecraft::{InstallMinecraftParams, MinecraftApplicationError, MinecraftInstallService},
    },
    shared::housekeeping::ActivityGate,
};

/// `inner`, run under a lease of `gate` for the whole call.
pub(super) struct Leased<T: ?Sized> {
    inner: Arc<T>,
    gate: Arc<ActivityGate>,
}

impl<T: ?Sized> Leased<T> {
    pub(super) fn new(inner: Arc<T>, gate: Arc<ActivityGate>) -> Self {
        Self { inner, gate }
    }
}

#[async_trait]
impl JavaInstallService for Leased<dyn JavaInstallService> {
    async fn execute(&self, install_java: InstallJava) -> Result<Java, JavaApplicationError> {
        let _lease = self.gate.lease().await;
        self.inner.execute(install_java).await
    }
}

#[async_trait]
impl MinecraftInstallService for Leased<dyn MinecraftInstallService> {
    async fn execute(
        &self,
        params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        let _lease = self.gate.lease().await;
        self.inner.execute(params, loading_bar, force).await
    }
}

#[async_trait]
impl InstallContentUseCasePort for Leased<dyn InstallContentUseCasePort> {
    async fn execute(&self, install_params: ContentInstallParams) -> Result<(), InstanceError> {
        let _lease = self.gate.lease().await;
        self.inner.execute(install_params).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::oneshot;

    use super::*;
    use crate::features::minecraft::ModLoader;

    /// Waits for `release` before it finishes; tells `started` when it has begun.
    struct Waiting {
        started: tokio::sync::mpsc::UnboundedSender<()>,
        release: tokio::sync::Mutex<Option<oneshot::Receiver<()>>>,
    }

    #[async_trait]
    impl MinecraftInstallService for Waiting {
        async fn execute(
            &self,
            _: InstallMinecraftParams,
            _: Option<&ProgressBarId>,
            _: bool,
        ) -> Result<(), MinecraftApplicationError> {
            self.started.send(()).unwrap();
            let release = self.release.lock().await.take().unwrap();
            let _ = release.await;
            Ok(())
        }
    }

    struct Fixture {
        gate: Arc<ActivityGate>,
        leased: Arc<Leased<dyn MinecraftInstallService>>,
        started: tokio::sync::mpsc::UnboundedReceiver<()>,
        release: oneshot::Sender<()>,
    }

    fn fixture() -> Fixture {
        let gate = Arc::new(ActivityGate::new());
        let (started_tx, started) = tokio::sync::mpsc::unbounded_channel();
        let (release, release_rx) = oneshot::channel();
        let inner: Arc<dyn MinecraftInstallService> = Arc::new(Waiting {
            started: started_tx,
            release: tokio::sync::Mutex::new(Some(release_rx)),
        });
        Fixture {
            leased: Arc::new(Leased::new(inner, gate.clone())),
            gate,
            started,
            release,
        }
    }

    async fn install(
        leased: Arc<Leased<dyn MinecraftInstallService>>,
    ) -> Result<(), MinecraftApplicationError> {
        let params = InstallMinecraftParams {
            game_version: "1.21.1".into(),
            loader: ModLoader::Vanilla,
            loader_version: None,
            install_dir: std::path::PathBuf::new(),
            java_path: None,
        };
        leased.execute(params, None, false).await
    }

    #[tokio::test]
    async fn should_keep_the_gate_busy_when_the_inner_call_runs() {
        let Fixture {
            gate,
            leased,
            mut started,
            release,
        } = fixture();

        let task = tokio::spawn(async move { install(leased).await });
        started.recv().await.unwrap();
        assert!(gate.try_pause().is_none());

        release.send(()).unwrap();
        task.await.unwrap().unwrap();
        assert!(gate.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_not_start_the_inner_call_when_the_gate_is_paused() {
        let Fixture {
            gate,
            leased,
            mut started,
            release,
        } = fixture();
        let pause = gate.try_pause().unwrap();

        let task = tokio::spawn(async move { install(leased).await });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(started.try_recv().is_err());

        drop(pause);
        tokio::time::timeout(Duration::from_secs(1), started.recv())
            .await
            .unwrap()
            .unwrap();
        release.send(()).unwrap();
        task.await.unwrap().unwrap();
    }
}
