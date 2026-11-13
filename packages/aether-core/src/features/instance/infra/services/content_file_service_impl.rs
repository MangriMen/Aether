use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{
            app::{ContentFileService, DISABLED_SUFFIX, LocatedContentFile},
            domain::{ContentPath, InstanceError, InstanceId, StorageError},
        },
        settings::LocationInfo,
    },
    shared::io::{
        IoError,
        infra::{create_dir_all, remove_file, rename},
    },
};

use super::instance_relative_path::resolve_instance_file_path;

/// Where `content_path` is on disk, resolved inside the instance directory (symlinks).
fn resolve_content(
    instance_id: &InstanceId,
    instance_dir: &Path,
    content_path: &ContentPath,
) -> Result<PathBuf, InstanceError> {
    resolve_instance_file_path(instance_id, instance_dir, content_path.as_relative())
}

/// Where the [`DISABLED_SUFFIX`] variant of `content_path` is on disk, resolved as
/// [`resolve_content`].
fn resolve_disabled(
    instance_id: &InstanceId,
    instance_dir: &Path,
    content_path: &ContentPath,
) -> Result<PathBuf, InstanceError> {
    resolve_instance_file_path(
        instance_id,
        instance_dir,
        &content_path.with_suffix(DISABLED_SUFFIX)?,
    )
}

pub struct FsContentFileService {
    location_info: Arc<LocationInfo>,
}

impl FsContentFileService {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    /// The directory of `instance_id`, or `None` when it has none: then no file of it can be
    /// removed, enabled or disabled.
    async fn existing_instance_dir(&self, instance_id: &InstanceId) -> Option<PathBuf> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        tokio::fs::metadata(&instance_dir)
            .await
            .is_ok_and(|metadata| metadata.is_dir())
            .then_some(instance_dir)
    }
}

#[async_trait]
impl ContentFileService for FsContentFileService {
    async fn remove_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(());
        };

        for content_path in content_paths {
            let enabled_path = resolve_content(instance_id, &instance_dir, content_path)?;
            // A disabled file is removed the same as an enabled one.
            let file_path = if enabled_path.exists() {
                enabled_path
            } else {
                resolve_disabled(instance_id, &instance_dir, content_path)?
            };

            match remove_file(&file_path).await {
                Ok(()) => {}
                // Neither the file nor its disabled variant is there: nothing left to remove.
                Err(IoError::IoPathError { source, .. })
                    if source.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(InstanceError::storage(err)),
            }
        }

        Ok(())
    }

    async fn enable_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(());
        };

        for content_path in content_paths {
            let enabled_path = resolve_content(instance_id, &instance_dir, content_path)?;

            if enabled_path.exists() {
                continue;
            }

            let disabled_path = resolve_disabled(instance_id, &instance_dir, content_path)?;

            if !disabled_path.exists() {
                continue;
            }

            rename(disabled_path, enabled_path)
                .await
                .map_err(InstanceError::storage)?;
        }

        Ok(())
    }

    async fn disable_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(());
        };

        for content_path in content_paths {
            let disabled_path = resolve_disabled(instance_id, &instance_dir, content_path)?;

            if disabled_path.exists() {
                continue;
            }

            let enabled_path = resolve_content(instance_id, &instance_dir, content_path)?;

            if !enabled_path.exists() {
                continue;
            }

            rename(enabled_path, disabled_path)
                .await
                .map_err(InstanceError::storage)?;
        }

        Ok(())
    }

    async fn locate_content_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<Option<LocatedContentFile>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        for disabled in [false, true] {
            let path = if disabled {
                resolve_disabled(instance_id, &instance_dir, content_path)?
            } else {
                resolve_content(instance_id, &instance_dir, content_path)?
            };

            if tokio::fs::metadata(&path)
                .await
                .is_ok_and(|metadata| metadata.is_file())
            {
                return Ok(Some(LocatedContentFile { path, disabled }));
            }
        }

        Ok(None)
    }

    async fn move_content_file_out(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        destination: &Path,
    ) -> Result<bool, InstanceError> {
        let content_path = &ContentPath::parse(content_path)?;
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(false);
        };
        let path = resolve_content(instance_id, &instance_dir, content_path)?;

        // Looked at first, so that a rename failing for want of `destination` is not taken for
        // an empty path.
        match tokio::fs::symlink_metadata(&path).await {
            // Whatever the user keeps in a directory at a content path is not a content file.
            Ok(metadata) if metadata.is_dir() => {
                return Err(InstanceError::storage(StorageError::other(format!(
                    "\"{content_path}\" of \"{instance_id}\" is a directory, not a content file"
                ))));
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => {
                return Err(InstanceError::storage(IoError::with_path(err, &path)));
            }
        }

        rename(&path, destination)
            .await
            .map(|()| true)
            .map_err(InstanceError::storage)
    }

    async fn install_content_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        temp_path: &Path,
    ) -> Result<(), InstanceError> {
        // The path may come from a provider's plan: it gets the same checks as a plugin's
        // `write_instance_file`, before and after the parent chain is created.
        let content_path = &ContentPath::parse(content_path)?;

        let instance_dir = self.location_info.instance_dir(instance_id);
        let absolute_content_path = resolve_content(instance_id, &instance_dir, content_path)?;

        if let Some(parent) = absolute_content_path.parent() {
            create_dir_all(parent)
                .await
                .map_err(InstanceError::storage)?;
        }

        let absolute_content_path = resolve_content(instance_id, &instance_dir, content_path)?;

        rename(temp_path, absolute_content_path)
            .await
            .map_err(InstanceError::storage)
    }
}
