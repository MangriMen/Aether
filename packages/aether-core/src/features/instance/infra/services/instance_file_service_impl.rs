use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use tokio::fs;
use tracing::warn;

use crate::{
    features::{
        instance::{
            app::InstanceFileService,
            domain::{InstanceError, InstanceId},
        },
        settings::LocationInfo,
    },
    shared::io::{
        domain::IoError,
        infra::{create_dir_all, remove_dir_all, rename},
    },
};

use super::instance_relative_path::{check_size, resolve_instance_file_path};

// On Windows a directory cannot be moved while a handle into it is still open, and the watcher
// and a just-stopped process release theirs a moment after being told to.
const MOVE_ATTEMPTS: u32 = 8;
const MOVE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(250);

/// Whether a failed move may succeed once whoever holds the directory lets go of it.
///
/// Only the Windows access-denied, sharing and lock violations (5, 32, 33) qualify; anything else
/// (a missing path, a real permission problem) would only make the caller wait for nothing.
fn is_held_open(err: &IoError) -> bool {
    let (IoError::IoPathError { source, .. } | IoError::IoError(source)) = err else {
        return false;
    };
    cfg!(windows) && matches!(source.raw_os_error(), Some(5 | 32 | 33))
}

async fn retry<T, E, Fut: std::future::Future<Output = Result<T, E>>>(
    attempts: u32,
    delay: std::time::Duration,
    is_transient: impl Fn(&E) -> bool,
    mut operation: impl FnMut() -> Fut,
) -> Result<T, E> {
    let mut attempt = 1;
    loop {
        match operation().await {
            Err(err) if attempt < attempts && is_transient(&err) => {
                attempt += 1;
                tokio::time::sleep(delay).await;
            }
            result => return result,
        }
    }
}

pub struct FsInstanceFileService {
    location_info: Arc<LocationInfo>,
    max_file_bytes: u64,
}

impl FsInstanceFileService {
    /// `max_file_bytes` bounds one file written or read through this service
    /// (`PluginSandboxLimits::max_instance_file_bytes`).
    pub fn new(location_info: Arc<LocationInfo>, max_file_bytes: u64) -> Self {
        Self {
            location_info,
            max_file_bytes,
        }
    }

    /// Deletes what an earlier removal moved aside but could not delete.
    ///
    /// Meant to run once at startup, when no removal is in flight. The trash lives outside the
    /// instances directory, so a leftover is never mistaken for an instance in the meantime.
    pub async fn sweep_removed(&self) {
        let trash_root = self.location_info.removed_instances_dir();
        let mut entries = match fs::read_dir(&trash_root).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
            Err(err) => {
                warn!("Failed to read {trash_root:?} to sweep removed instances: {err}");
                return;
            }
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Err(err) = remove_dir_all(entry.path()).await {
                warn!("Failed to sweep removed instance {:?}: {err}", entry.path());
            }
        }
    }
}

#[async_trait]
impl InstanceFileService for FsInstanceFileService {
    async fn create_instance_dir(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        create_dir_all(&instance_dir)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    async fn create_unique_instance_dir(
        &self,
        base_id: &InstanceId,
    ) -> Result<(InstanceId, PathBuf), InstanceError> {
        let instances_root = self.location_info.instances_dir();

        // Ensure the instances root directory exists first
        create_dir_all(&instances_root)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        let mut counter = 0u32;
        loop {
            // Parsed before the directory exists: an id that cannot name one must not leave one.
            let id = if counter == 0 {
                base_id.clone()
            } else {
                InstanceId::try_from(format!("{base_id}-{counter}"))?
            };
            let path = instances_root.join(id.as_str());
            match fs::create_dir(&path).await {
                Ok(()) => return Ok((id, path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    counter += 1;
                }
                Err(e) => {
                    return Err(InstanceError::Storage(format!(
                        "Failed to create instance directory '{}': {e}",
                        path.display()
                    )));
                }
            }
        }
    }

    /// Moves the directory out of the way first, then deletes it. The move is all or nothing: if
    /// something holds a file open, the instance stays whole and the removal can be retried, rather
    /// than being left half deleted. Once moved, the instance is gone; a failed delete only leaves
    /// a directory in the trash, which [`FsInstanceFileService::sweep_removed`] deletes at startup.
    async fn remove_instance_dir(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        if !instance_dir.exists() {
            return Ok(());
        }

        let trash_root = self.location_info.removed_instances_dir();
        create_dir_all(&trash_root)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        let trash = trash_root.join(uuid::Uuid::new_v4().simple().to_string());

        retry(MOVE_ATTEMPTS, MOVE_RETRY_DELAY, is_held_open, || {
            rename(&instance_dir, &trash)
        })
        .await
        .map_err(|err| InstanceError::Storage(err.to_string()))?;

        if let Err(err) = remove_dir_all(&trash).await {
            warn!("Removed instance {instance_id} but could not delete {trash:?}: {err}");
        }
        Ok(())
    }

    async fn write_instance_file(
        &self,
        instance_id: &InstanceId,
        relative_path: &str,
        bytes: &[u8],
    ) -> Result<(), InstanceError> {
        check_size(relative_path, bytes.len() as u64, self.max_file_bytes)?;

        let instance_dir = self.location_info.instance_dir(instance_id);
        let target = resolve_instance_file_path(instance_id, &instance_dir, relative_path)?;

        if let Some(parent) = target.parent() {
            create_dir_all(parent)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        // Resolve a second time, now that the parent chain exists. `create_dir_all` only ever
        // creates plain directories, so this cannot newly pass — but it can newly fail, if
        // something replaced a component with a symlink since the first check.
        let target = resolve_instance_file_path(instance_id, &instance_dir, relative_path)?;

        fs::write(&target, bytes).await.map_err(|err| {
            InstanceError::Storage(format!("Failed to write \"{}\": {err}", target.display()))
        })
    }

    async fn read_instance_file(
        &self,
        instance_id: &InstanceId,
        relative_path: &str,
    ) -> Result<Vec<u8>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        let target = resolve_instance_file_path(instance_id, &instance_dir, relative_path)?;

        let metadata = fs::metadata(&target).await.map_err(|err| {
            InstanceError::Storage(format!("Failed to read \"{}\": {err}", target.display()))
        })?;

        if !metadata.is_file() {
            return Err(InstanceError::InvalidRelativePath {
                path: relative_path.to_owned(),
                reason: "path is not a regular file".to_owned(),
            });
        }

        // Checked from the metadata rather than after reading, so an oversized file is never
        // pulled into memory at all.
        check_size(relative_path, metadata.len(), self.max_file_bytes)?;

        fs::read(&target).await.map_err(|err| {
            InstanceError::Storage(format!("Failed to read \"{}\": {err}", target.display()))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use super::retry;

    #[tokio::test]
    async fn should_retry_until_the_operation_succeeds_when_it_fails_transiently() {
        let calls = AtomicU32::new(0);

        let result: Result<u32, &str> = retry(
            5,
            Duration::ZERO,
            |_| true,
            || async {
                match calls.fetch_add(1, Ordering::SeqCst) {
                    0..=1 => Err("locked"),
                    n => Ok(n),
                }
            },
        )
        .await;

        assert_eq!(result, Ok(2));
    }

    #[tokio::test]
    async fn should_return_the_last_error_when_every_attempt_fails() {
        let calls = AtomicU32::new(0);

        let result: Result<(), &str> = retry(
            3,
            Duration::ZERO,
            |_| true,
            || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err("locked")
            },
        )
        .await;

        assert_eq!(result, Err("locked"));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn should_not_retry_when_the_error_is_not_transient() {
        let calls = AtomicU32::new(0);

        let result: Result<(), &str> = retry(
            5,
            Duration::ZERO,
            |_| false,
            || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err("missing")
            },
        )
        .await;

        assert_eq!(result, Err("missing"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
