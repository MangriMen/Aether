use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{
            app::{ContentFileService, DISABLED_SUFFIX, LocatedContentFile},
            domain::InstanceError,
        },
        settings::LocationInfo,
    },
    shared::io::{
        IoError,
        infra::{create_dir_all, remove_file, rename},
    },
};

use super::instance_relative_path::{check_content_path, resolve_instance_file_path};

pub struct FsContentFileService {
    location_info: Arc<LocationInfo>,
}

impl FsContentFileService {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    /// The directory of `instance_id`, or `None` when it has none: then no file of it can be
    /// removed, enabled or disabled.
    async fn existing_instance_dir(&self, instance_id: &str) -> Option<PathBuf> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        tokio::fs::metadata(&instance_dir)
            .await
            .is_ok_and(|metadata| metadata.is_dir())
            .then_some(instance_dir)
    }
}

/// The paths come from the pack storage and from the UI, which are no more trusted than a plan:
/// each is checked as one before any file is touched. A path that fails is skipped with a warning
/// — nothing is done to it — rather than failing the rest: a stale record or a name this platform
/// refuses must not keep the user from changing the other files. A symlink out of the instance is
/// only found as each file is reached.
fn valid_content_paths<'a>(instance_id: &str, content_paths: &'a [String]) -> Vec<&'a str> {
    content_paths
        .iter()
        .filter_map(|content_path| match check_content_path(content_path) {
            Ok(_) => Some(content_path.as_str()),
            Err(err) => {
                tracing::warn!("Skipping \"{content_path}\" of \"{instance_id}\": {err}");
                None
            }
        })
        .collect()
}

#[async_trait]
impl ContentFileService for FsContentFileService {
    async fn remove_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let content_paths = valid_content_paths(instance_id, content_paths);
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(());
        };

        for content_path in content_paths {
            let mut file_path =
                resolve_instance_file_path(instance_id, &instance_dir, content_path)?;

            if !file_path.exists() {
                // TODO rework disable logic to make it more transparent
                file_path = resolve_instance_file_path(
                    instance_id,
                    &instance_dir,
                    &format!("{content_path}{DISABLED_SUFFIX}"),
                )?;
            }

            match remove_file(&file_path).await {
                Ok(()) => {}
                // Neither the file nor its disabled variant is there: nothing left to remove.
                Err(IoError::IoPathError { source, .. })
                    if source.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(InstanceError::Storage(err.to_string())),
            }
        }

        Ok(())
    }

    async fn enable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let content_paths = valid_content_paths(instance_id, content_paths);
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(());
        };

        for content_path in content_paths {
            let enabled_path =
                resolve_instance_file_path(instance_id, &instance_dir, content_path)?;

            if enabled_path.exists() {
                continue;
            }

            let disabled_path = resolve_instance_file_path(
                instance_id,
                &instance_dir,
                &format!("{content_path}{DISABLED_SUFFIX}"),
            )?;

            if !disabled_path.exists() {
                continue;
            }

            rename(disabled_path, enabled_path)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    async fn disable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let content_paths = valid_content_paths(instance_id, content_paths);
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(());
        };

        for content_path in content_paths {
            let disabled_path = resolve_instance_file_path(
                instance_id,
                &instance_dir,
                &format!("{content_path}{DISABLED_SUFFIX}"),
            )?;

            if disabled_path.exists() {
                continue;
            }

            let enabled_path =
                resolve_instance_file_path(instance_id, &instance_dir, content_path)?;

            if !enabled_path.exists() {
                continue;
            }

            rename(enabled_path, disabled_path)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    async fn locate_content_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<Option<LocatedContentFile>, InstanceError> {
        check_content_path(content_path)?;

        let instance_dir = self.location_info.instance_dir(instance_id);

        for (candidate, disabled) in [
            (content_path.to_owned(), false),
            (format!("{content_path}{DISABLED_SUFFIX}"), true),
        ] {
            let path = resolve_instance_file_path(instance_id, &instance_dir, &candidate)?;

            if tokio::fs::metadata(&path)
                .await
                .is_ok_and(|metadata| metadata.is_file())
            {
                return Ok(Some(LocatedContentFile { path, disabled }));
            }
        }

        Ok(None)
    }

    fn check_content_path(&self, content_path: &str) -> Result<(), InstanceError> {
        check_content_path(content_path).map(drop)
    }

    async fn move_content_file_out(
        &self,
        instance_id: &str,
        content_path: &str,
        destination: &Path,
    ) -> Result<bool, InstanceError> {
        check_content_path(content_path)?;
        let Some(instance_dir) = self.existing_instance_dir(instance_id).await else {
            return Ok(false);
        };
        let path = resolve_instance_file_path(instance_id, &instance_dir, content_path)?;

        // Looked at first, so that a rename failing for want of `destination` is not taken for
        // an empty path.
        match tokio::fs::symlink_metadata(&path).await {
            // Whatever the user keeps in a directory at a content path is not a content file.
            Ok(metadata) if metadata.is_dir() => {
                return Err(InstanceError::Storage(format!(
                    "\"{content_path}\" of \"{instance_id}\" is a directory, not a content file"
                )));
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => {
                return Err(InstanceError::Storage(
                    IoError::with_path(err, &path).to_string(),
                ));
            }
        }

        rename(&path, destination)
            .await
            .map(|()| true)
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    async fn install_content_file(
        &self,
        instance_id: &str,
        content_path: &str,
        temp_path: &Path,
    ) -> Result<(), InstanceError> {
        // The path may come from a provider's plan: it gets the same checks as a plugin's
        // `write_instance_file`, before and after the parent chain is created.
        check_content_path(content_path)?;

        let instance_dir = self.location_info.instance_dir(instance_id);
        let absolute_content_path =
            resolve_instance_file_path(instance_id, &instance_dir, content_path)?;

        if let Some(parent) = absolute_content_path.parent() {
            create_dir_all(parent)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        let absolute_content_path =
            resolve_instance_file_path(instance_id, &instance_dir, content_path)?;

        rename(temp_path, absolute_content_path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }
}
