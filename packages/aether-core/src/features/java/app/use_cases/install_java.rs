use std::{path::PathBuf, sync::Arc};

use crate::features::events::{ProgressBarId, ProgressService, SharedDownload};
use crate::features::java::{
    Java, JavaApplicationError, JavaInstallService, JavaInstallationService,
    JavaInstallationTracker, JavaStorage, JreProvider, app::InstallJava,
};
use crate::features::settings::LocationInfo;

pub struct InstallJavaUseCase {
    storage: Arc<dyn JavaStorage>,
    java_installation_service: Arc<dyn JavaInstallationService>,
    provider: Arc<dyn JreProvider>,
    location_info: Arc<LocationInfo>,
    java_installation_tracker: Arc<dyn JavaInstallationTracker>,
    progress_service: Arc<dyn ProgressService>,
}

#[async_trait::async_trait]
impl JavaInstallService for InstallJavaUseCase {
    async fn execute(
        &self,
        install_java: InstallJava,
        waiting_bar: Option<&ProgressBarId>,
    ) -> Result<Java, JavaApplicationError> {
        let InstallJava { version, force } = install_java;

        // Before the lock: a download another install runs under it is mirrored too. The
        // provider starts the shared bar of the download.
        let _wait = self
            .progress_service
            .wait_on_shared(waiting_bar, SharedDownload::Java { version })
            .await;
        let _guard = self.java_installation_tracker.lock(version).await;

        if !force
            && let Ok(Some(existing_java)) = self.storage.get(version).await
            && self
                .java_installation_service
                .locate_java(&PathBuf::from(existing_java.path().to_owned()))
                .await
                .is_ok()
        {
            return Ok(existing_java);
        }

        let installed_jre_path = self
            .provider
            .install(version, &self.location_info.java_dir())
            .await?;

        let java = self
            .java_installation_service
            .locate_java(&installed_jre_path)
            .await?;

        Ok(self.storage.upsert(java).await?)
    }
}

impl InstallJavaUseCase {
    pub fn new(
        storage: Arc<dyn JavaStorage>,
        java_installation_service: Arc<dyn JavaInstallationService>,
        provider: Arc<dyn JreProvider>,
        location_info: Arc<LocationInfo>,
        java_installation_tracker: Arc<dyn JavaInstallationTracker>,
        progress_service: Arc<dyn ProgressService>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
            provider,
            location_info,
            java_installation_tracker,
            progress_service,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use async_trait::async_trait;
    use tokio::sync::{Notify, mpsc};

    use crate::features::events::{
        Event, ProgressBarStorage, ProgressServiceExt, ProgressServiceImpl, ProgressStatus,
        infra::InMemoryProgressBarStorage,
        test_support::{ChannelEmitter, mirrors, pack, sent},
    };
    use crate::features::java::{JavaDomainError, infra::MemoryJavaInstallationTracker};

    use super::*;

    #[derive(Default)]
    struct MemStorage(Mutex<HashMap<u32, Java>>);

    #[async_trait]
    impl JavaStorage for MemStorage {
        async fn list(&self) -> Result<Vec<Java>, JavaDomainError> {
            Ok(self.0.lock().unwrap().values().cloned().collect())
        }

        async fn get(&self, version: u32) -> Result<Option<Java>, JavaDomainError> {
            Ok(self.0.lock().unwrap().get(&version).cloned())
        }

        async fn upsert(&self, java: Java) -> Result<Java, JavaDomainError> {
            self.0
                .lock()
                .unwrap()
                .insert(java.major_version(), java.clone());
            Ok(java)
        }

        async fn remove(&self, version: u32) -> Result<(), JavaDomainError> {
            self.0.lock().unwrap().remove(&version);
            Ok(())
        }
    }

    /// Takes any path for a Java 21.
    struct AnyJava;

    #[async_trait]
    impl JavaInstallationService for AnyJava {
        async fn locate_java(&self, path: &Path) -> Result<Java, JavaDomainError> {
            Ok(Java::new(
                21,
                "21.0.1".into(),
                "x64".into(),
                path.display().to_string(),
            ))
        }

        async fn discover_installations(
            &self,
            _: &[PathBuf],
        ) -> Result<Vec<Java>, JavaDomainError> {
            Ok(Vec::new())
        }
    }

    /// Downloads under a shared bar, as the real provider does, once `release` is notified.
    struct GatedProvider {
        progress: Arc<dyn ProgressService>,
        installs: AtomicUsize,
        release: Notify,
    }

    #[async_trait]
    impl JreProvider for GatedProvider {
        async fn install(
            &self,
            version: u32,
            install_dir: &Path,
        ) -> Result<PathBuf, JavaDomainError> {
            self.installs.fetch_add(1, Ordering::SeqCst);
            let bar = self
                .progress
                .init_shared_progress_safe(
                    SharedDownload::Java { version },
                    100.0,
                    "Downloading java version".into(),
                )
                .await
                .unwrap();
            self.progress
                .add_progress_bytes_total_safe(&bar, 1000)
                .await;
            self.release.notified().await;
            self.progress.add_progress_bytes_safe(&bar, 1000).await;
            self.progress.emit_progress_safe(&bar, 100.0, None).await;
            Ok(install_dir.join("jre-21"))
        }
    }

    struct Fixture {
        use_case: Arc<InstallJavaUseCase>,
        progress: Arc<ProgressServiceImpl<InMemoryProgressBarStorage>>,
        storage: Arc<InMemoryProgressBarStorage>,
        provider: Arc<GatedProvider>,
        java_storage: Arc<MemStorage>,
        events: mpsc::UnboundedReceiver<Event>,
        _temp: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let temp = tempfile::TempDir::new().unwrap();
        let (tx, events) = mpsc::unbounded_channel();
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let progress = Arc::new(ProgressServiceImpl::new(
            Arc::new(ChannelEmitter(tx)),
            storage.clone(),
        ));
        let provider = Arc::new(GatedProvider {
            progress: progress.clone(),
            installs: AtomicUsize::new(0),
            release: Notify::new(),
        });
        let java_storage = Arc::new(MemStorage::default());
        let use_case = Arc::new(InstallJavaUseCase::new(
            java_storage.clone(),
            Arc::new(AnyJava),
            provider.clone(),
            Arc::new(LocationInfo::new(
                temp.path().join("settings"),
                temp.path().to_path_buf(),
            )),
            Arc::new(MemoryJavaInstallationTracker::default()),
            progress.clone(),
        ));
        Fixture {
            use_case,
            progress,
            storage,
            provider,
            java_storage,
            events,
            _temp: temp,
        }
    }

    #[tokio::test]
    async fn should_download_once_and_mirror_it_for_both_when_two_installs_need_one_java() {
        let mut fixture = fixture();
        let a = fixture
            .progress
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();
        let b = fixture
            .progress
            .init_progress(pack("b"), 1.0, String::new())
            .await
            .unwrap();
        let (a_id, b_id) = (a.id, b.id);

        let first = tokio::spawn({
            let use_case = fixture.use_case.clone();
            async move {
                let java = use_case.execute(InstallJava::new(21), Some(&a)).await;
                (java, a)
            }
        });
        let [(_, shared)] = mirrors(&fixture.storage, 1).await[..] else {
            panic!("one mirror expected");
        };
        let second = tokio::spawn({
            let use_case = fixture.use_case.clone();
            async move {
                let java = use_case.execute(InstallJava::new(21), Some(&b)).await;
                (java, b)
            }
        });

        let mut expected = vec![(a_id, shared), (b_id, shared)];
        expected.sort_unstable();
        assert_eq!(mirrors(&fixture.storage, 2).await, expected);
        let shared_bar = fixture.storage.get(shared).await.unwrap().clone();
        assert_eq!(shared_bar.parent_id, None, "the download is nobody's");

        fixture.provider.release.notify_one();
        let (first, a) = first.await.unwrap();
        let (second, b) = second.await.unwrap();
        assert_eq!(first.unwrap(), second.unwrap());
        assert_eq!(fixture.provider.installs.load(Ordering::SeqCst), 1);

        // The packs count none of the shared bytes.
        for pack in [&a, &b] {
            let bar = fixture.storage.get(pack.id).await.unwrap().clone();
            assert_eq!((bar.bytes_done, bar.bytes_total), (None, None));
        }
        drop((a, b));
        tokio::time::timeout(Duration::from_secs(5), async {
            while !fixture.storage.list().await.is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let ended: Vec<_> = sent(&mut fixture.events)
            .into_iter()
            .filter(|event| {
                event.mirror_of == Some(shared) && event.status != ProgressStatus::Running
            })
            .map(|event| (event.status, event.bytes_done))
            .collect();
        assert_eq!(
            ended,
            [
                (ProgressStatus::Done, Some(1000)),
                (ProgressStatus::Done, Some(1000))
            ],
            "both mirrors end as the download did"
        );
    }

    #[tokio::test]
    async fn should_mirror_nothing_when_the_java_is_installed_already() {
        let fixture = fixture();
        fixture
            .java_storage
            .upsert(Java::new(21, "21".into(), "x64".into(), "/jre".into()))
            .await
            .unwrap();
        let a = fixture
            .progress
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();

        fixture
            .use_case
            .execute(InstallJava::new(21), Some(&a))
            .await
            .unwrap();

        let bars: Vec<_> = fixture
            .storage
            .list()
            .await
            .into_iter()
            .map(|bar| bar.id)
            .collect();
        assert_eq!(bars, [a.id]);
        assert_eq!(fixture.provider.installs.load(Ordering::SeqCst), 0);
    }
}
