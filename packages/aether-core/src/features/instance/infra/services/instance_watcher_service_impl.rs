use std::{path::Path, sync::Arc};

use async_trait::async_trait;

use crate::features::{
    file_watcher::FileWatcher,
    instance::{
        app::InstanceWatcherService,
        domain::{ContentType, InstanceError},
    },
    settings::LocationInfo,
};

const CRASH_REPORTS_FOLDER: &str = "crash-reports";

pub struct InstanceWatcherServiceImpl<FW: FileWatcher> {
    file_watcher: Arc<FW>,
    location_info: Arc<LocationInfo>,
}

impl<FW: FileWatcher> InstanceWatcherServiceImpl<FW> {
    pub fn new(file_watcher: Arc<FW>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            file_watcher,
            location_info,
        }
    }
}

/// Folders inside an instance whose changes are reported: every content folder plus the
/// directory where the game writes its crash reports.
fn watched_subfolders() -> impl Iterator<Item = &'static str> {
    ContentType::iterator()
        .map(|content| content.get_folder())
        .chain(std::iter::once(CRASH_REPORTS_FOLDER))
}

impl<FW: FileWatcher> InstanceWatcherServiceImpl<FW> {
    /// Creates the folder when nothing (not even a dangling symlink) occupies the path,
    /// then starts observing it.
    async fn ensure_and_watch(&self, path: &Path) -> Result<(), String> {
        if tokio::fs::symlink_metadata(path).await.is_err() {
            tokio::fs::create_dir_all(path)
                .await
                .map_err(|e| format!("cannot create it: {e}"))?;
        }

        self.file_watcher
            .watch(path)
            .await
            .map_err(|e| format!("watcher refused it: {e}"))
    }
}

#[async_trait]
impl<FW: FileWatcher> InstanceWatcherService for InstanceWatcherServiceImpl<FW> {
    async fn watch_instances(&self) -> Result<(), InstanceError> {
        if let Ok(instances_dir) = std::fs::read_dir(self.location_info.instances_dir()) {
            for instance_dir in instances_dir {
                if let Ok(file_name) = instance_dir.map(|x| x.file_name())
                    && let Some(file_name) = file_name.to_str()
                {
                    if file_name.starts_with(".DS_Store") {
                        continue;
                    }

                    let _ = self.watch_instance(file_name).await;
                }
            }
        }

        Ok(())
    }

    async fn watch_instance(&self, instance_id: &str) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        if !instance_dir.is_dir() {
            return Ok(());
        }

        // A failure on one folder must not leave the remaining ones unobserved.
        for folder in watched_subfolders() {
            let folder_path = instance_dir.join(folder);
            if let Err(reason) = self.ensure_and_watch(&folder_path).await {
                tracing::error!("Instance folder {folder_path:?} is not being watched: {reason}");
            }
        }

        Ok(())
    }

    async fn unwatch_instance(&self, instance_id: &str) -> Result<(), InstanceError> {
        let path = self.location_info.instance_dir(instance_id);
        if let Err(reason) = self.file_watcher.unwatch(&path).await {
            tracing::error!("Could not stop watching {path:?}: {reason}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use tempfile::TempDir;

    use crate::features::file_watcher::FileWatcherError;

    use super::*;

    #[derive(Default)]
    struct RecordingWatcher {
        watched: Mutex<Vec<PathBuf>>,
        unwatched: Mutex<Vec<PathBuf>>,
        fail_on: Option<PathBuf>,
    }

    #[async_trait]
    impl FileWatcher for RecordingWatcher {
        async fn watch(&self, path: &Path) -> Result<(), FileWatcherError> {
            if self.fail_on.as_deref() == Some(path) {
                return Err(FileWatcherError::WatchNotFound);
            }
            self.watched.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }

        async fn unwatch(&self, path: &Path) -> Result<(), FileWatcherError> {
            self.unwatched.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    fn setup() -> (
        TempDir,
        Arc<RecordingWatcher>,
        Arc<LocationInfo>,
        InstanceWatcherServiceImpl<RecordingWatcher>,
    ) {
        let temp = TempDir::new().unwrap();
        let location = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        ));
        let watcher = Arc::new(RecordingWatcher::default());
        let service = InstanceWatcherServiceImpl::new(watcher.clone(), location.clone());
        (temp, watcher, location, service)
    }

    #[tokio::test]
    async fn watch_instance_creates_and_watches_content_and_crash_folders() {
        let (_temp, watcher, location, service) = setup();
        let dir = location.instance_dir("inst");
        std::fs::create_dir_all(&dir).unwrap();

        service.watch_instance("inst").await.unwrap();

        let expected: Vec<PathBuf> = [
            "mods",
            "datapacks",
            "resourcepacks",
            "shaderpacks",
            "crash-reports",
        ]
        .iter()
        .map(|f| dir.join(f))
        .collect();
        assert_eq!(*watcher.watched.lock().unwrap(), expected);
        assert!(expected.iter().all(|p| p.is_dir()));
    }

    #[tokio::test]
    async fn watch_instance_keeps_watching_other_folders_after_a_failure() {
        let temp = TempDir::new().unwrap();
        let location = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        ));
        let dir = location.instance_dir("inst");
        std::fs::create_dir_all(&dir).unwrap();
        let watcher = Arc::new(RecordingWatcher {
            fail_on: Some(dir.join("datapacks")),
            ..Default::default()
        });
        let service = InstanceWatcherServiceImpl::new(watcher.clone(), location);

        service.watch_instance("inst").await.unwrap();

        let expected: Vec<PathBuf> = ["mods", "resourcepacks", "shaderpacks", "crash-reports"]
            .iter()
            .map(|f| dir.join(f))
            .collect();
        assert_eq!(*watcher.watched.lock().unwrap(), expected);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn watch_instance_does_not_replace_dangling_symlink() {
        let (temp, watcher, location, service) = setup();
        let dir = location.instance_dir("inst");
        std::fs::create_dir_all(&dir).unwrap();
        let link = dir.join("mods");
        std::os::unix::fs::symlink(temp.path().join("missing-target"), &link).unwrap();

        service.watch_instance("inst").await.unwrap();

        assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
        assert!(!link.exists());
        assert!(watcher.watched.lock().unwrap().contains(&link));
    }

    #[tokio::test]
    async fn watch_instance_ignores_missing_instance_dir() {
        let (_temp, watcher, location, service) = setup();

        service.watch_instance("absent").await.unwrap();

        assert!(watcher.watched.lock().unwrap().is_empty());
        assert!(!location.instance_dir("absent").exists());
    }

    #[tokio::test]
    async fn watch_instances_skips_ds_store_entries() {
        let (_temp, recorder, location, service) = setup();
        std::fs::create_dir_all(location.instance_dir("a")).unwrap();
        std::fs::create_dir_all(location.instance_dir(".DS_Store")).unwrap();

        service.watch_instances().await.unwrap();

        let seen = recorder.watched.lock().unwrap();
        assert_eq!(seen.len(), 5);
        assert!(
            seen.iter()
                .all(|p| p.starts_with(location.instance_dir("a")))
        );
    }

    #[tokio::test]
    async fn unwatch_instance_unwatches_instance_root() {
        let (_temp, watcher, location, service) = setup();

        service.unwatch_instance("inst").await.unwrap();

        assert_eq!(
            *watcher.unwatched.lock().unwrap(),
            vec![location.instance_dir("inst")]
        );
    }
}
