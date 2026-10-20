use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::features::instance::app::ports::ImportContentUseCasePort;
use crate::{
    features::{
        events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
        instance::{
            ContentType, InstanceError, InstanceId, InstanceLocks, PackFile, PackStorage,
            app::DISABLED_SUFFIX,
        },
        settings::LocationInfo,
    },
    shared::{
        hash::infra::sha1_async,
        io::infra::{canonicalize, read_async},
    },
};

pub struct ImportContent {
    instance_id: InstanceId,
    content_type: ContentType,
    source_paths: Vec<PathBuf>,
}

impl ImportContent {
    pub fn new(
        instance_id: InstanceId,
        content_type: ContentType,
        source_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            instance_id,
            content_type,
            source_paths,
        }
    }
}

/// Imports files under the instance's lock ([`InstanceLocks`]): the duplicate check, the copy and
/// the records are one step no plan may interleave with.
pub struct ImportContentUseCase {
    event_emitter: SharedEventEmitter,
    pack_storage: Arc<dyn PackStorage>,
    location_info: Arc<LocationInfo>,
    instance_locks: Arc<InstanceLocks>,
}

impl ImportContentUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        pack_storage: Arc<dyn PackStorage>,
        location_info: Arc<LocationInfo>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            event_emitter,
            pack_storage,
            location_info,
            instance_locks,
        }
    }

    async fn prepare_import_data(
        &self,
        instance_id: &InstanceId,
        content_type: ContentType,
        source_paths: &[PathBuf],
    ) -> Result<(Vec<String>, Vec<PackFile>), InstanceError> {
        let mut paths = Vec::with_capacity(source_paths.len());
        let mut metadata_files = Vec::with_capacity(source_paths.len());
        let mut seen = HashSet::with_capacity(source_paths.len());

        for source_path in source_paths {
            let (content_path, metadata) = self
                .get_import_content_data(instance_id, content_type, source_path)
                .await?;

            // Two sources of one name would be copied onto each other; Windows and macOS
            // file systems treat names differing in case as one.
            let key = if cfg!(any(windows, target_os = "macos")) {
                content_path.to_lowercase()
            } else {
                content_path.clone()
            };
            if !seen.insert(key) {
                return Err(InstanceError::ContentDuplication { content_path });
            }

            paths.push(content_path);
            metadata_files.push(metadata);
        }

        Ok((paths, metadata_files))
    }

    async fn get_import_content_data(
        &self,
        instance_id: &InstanceId,
        content_type: ContentType,
        path: &Path,
    ) -> Result<(String, PackFile), InstanceError> {
        let content_folder = content_type.get_folder();

        let file_name =
            path.file_name()
                .and_then(|n| n.to_str())
                .ok_or(InstanceError::ContentFilename {
                    path: path.to_path_buf(),
                })?;

        // Always with `/`, as the pack records and the content listing key a file: a `\` here
        // on Windows would record a file the listing never finds, and import it again as new.
        let content_path = format!("{content_folder}/{file_name}");

        let instance_dir = self.location_info.instance_dir(instance_id);
        Self::ensure_folder_inside_instance(instance_id, &instance_dir, content_folder)?;

        let absolute_content_path = instance_dir.join(&content_path);
        let disabled_path =
            absolute_content_path.with_file_name(format!("{file_name}{DISABLED_SUFFIX}"));

        // `symlink_metadata`: a dangling link is still in the way of a copy, which follows it.
        if absolute_content_path.symlink_metadata().is_ok()
            || disabled_path.symlink_metadata().is_ok()
        {
            return Err(InstanceError::ContentDuplication {
                content_path: content_path.clone(),
            });
        }

        let pack_file = self
            .pack_storage
            .get_pack_file(instance_id, &content_path)
            .await;

        if pack_file.is_ok() {
            return Err(InstanceError::ContentDuplication {
                content_path: content_path.clone(),
            });
        }

        let file_content = read_async(path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        let hash = sha1_async(file_content)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        Ok((
            content_path,
            PackFile::from_hash(file_name.to_owned(), hash),
        ))
    }

    /// A `mods` folder that is a link to elsewhere would take the copy out of the instance.
    fn ensure_folder_inside_instance(
        instance_id: &InstanceId,
        instance_dir: &Path,
        content_folder: &str,
    ) -> Result<(), InstanceError> {
        let root =
            canonicalize(instance_dir).map_err(|err| InstanceError::Storage(err.to_string()))?;
        let folder = canonicalize(instance_dir.join(content_folder))
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        if folder.starts_with(&root) {
            Ok(())
        } else {
            Err(InstanceError::PathEscapesInstance {
                path: content_folder.to_owned(),
                instance_id: instance_id.to_string(),
            })
        }
    }

    async fn copy_import_files(
        &self,
        instance_id: &InstanceId,
        source_paths: &[PathBuf],
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        let mut copied = Vec::with_capacity(content_paths.len());

        for (src, dest) in source_paths.iter().zip(content_paths) {
            let dest_path = instance_dir.join(dest);
            if let Err(err) = tokio::fs::copy(src, &dest_path).await {
                // A half-written file is not in the way of another import, but is not
                // wanted either.
                copied.push(dest_path);
                Self::remove_files(&copied).await;
                return Err(InstanceError::Storage(err.to_string()));
            }
            copied.push(dest_path);
        }

        Ok(())
    }

    /// Only for files this import created: none existed under these names when it began, and
    /// the instance's lock keeps anything else from creating them.
    async fn remove_files(paths: &[PathBuf]) {
        for path in paths {
            if let Err(err) = tokio::fs::remove_file(path).await
                && err.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(
                    "Could not remove {} after a failed import: {err}",
                    path.display()
                );
            }
        }
    }

    pub async fn execute(&self, input: ImportContent) -> Result<(), InstanceError> {
        let ImportContent {
            instance_id,
            content_type,
            source_paths,
        } = input;

        if content_type == ContentType::Modpack {
            return Err(InstanceError::UnsupportedContentType { content_type });
        }

        let _lock = self.instance_locks.lock(&instance_id).await;

        let (content_paths, pack_files) = self
            .prepare_import_data(&instance_id, content_type, source_paths.as_slice())
            .await?;

        self.copy_import_files(&instance_id, source_paths.as_slice(), &content_paths)
            .await?;

        if let Err(err) = self
            .pack_storage
            .update_pack_file_many(&instance_id, &content_paths, &pack_files)
            .await
        {
            let instance_dir = self.location_info.instance_dir(&instance_id);
            let copied: Vec<PathBuf> = content_paths
                .iter()
                .map(|content_path| instance_dir.join(content_path))
                .collect();
            Self::remove_files(&copied).await;
            return Err(err);
        }

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Edited,
                instance_id: instance_id.clone(),
            })
            .await;

        Ok(())
    }
}

#[async_trait]
impl ImportContentUseCasePort for ImportContentUseCase {
    async fn execute(&self, input: ImportContent) -> Result<(), InstanceError> {
        self.execute(input).await
    }
}
