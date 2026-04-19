use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    features::{
        events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
        instance::{ContentType, InstanceError, PackFile, PackStorage},
        settings::LocationInfo,
    },
    shared::{IoError, read_async},
};

pub struct ImportContent {
    instance_id: String,
    content_type: ContentType,
    source_paths: Vec<PathBuf>,
}

impl ImportContent {
    pub fn new(instance_id: String, content_type: ContentType, source_paths: Vec<PathBuf>) -> Self {
        Self {
            instance_id,
            content_type,
            source_paths,
        }
    }
}

pub struct ImportContentUseCase<PS: PackStorage> {
    event_emitter: SharedEventEmitter,
    pack_storage: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<PS: PackStorage> ImportContentUseCase<PS> {
    pub fn new(
        event_emitter: SharedEventEmitter,
        pack_storage: Arc<PS>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            event_emitter,
            pack_storage,
            location_info,
        }
    }

    async fn prepare_import_data(
        &self,
        instance_id: &str,
        content_type: ContentType,
        source_paths: &[PathBuf],
    ) -> Result<(Vec<String>, Vec<PackFile>), InstanceError> {
        let mut paths = Vec::with_capacity(source_paths.len());
        let mut metadata_files = Vec::with_capacity(source_paths.len());

        for source_path in source_paths {
            let (content_path, metadata) = self
                .get_import_content_data(instance_id, content_type, source_path)
                .await?;

            paths.push(content_path);
            metadata_files.push(metadata);
        }

        Ok((paths, metadata_files))
    }

    async fn get_import_content_data(
        &self,
        instance_id: &str,
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

        let content_path = Path::new(content_folder)
            .join(file_name)
            .to_string_lossy()
            .to_string();

        let absolute_content_path = self
            .location_info
            .instance_dir(instance_id)
            .join(&content_path);

        if absolute_content_path.exists() {
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

        let file_content = read_async(path).await?;

        Ok((
            content_path,
            PackFile::from_contents(file_name.to_owned(), file_content).await,
        ))
    }

    async fn copy_import_files(
        &self,
        instance_id: &str,
        source_paths: &[PathBuf],
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        futures::future::try_join_all(source_paths.iter().zip(content_paths).map(|(src, dest)| {
            let dest_path = instance_dir.join(dest);
            tokio::fs::copy(src, dest_path)
        }))
        .await
        .map_err(IoError::from)?;

        Ok(())
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

        let (content_paths, pack_files) = self
            .prepare_import_data(&instance_id, content_type, source_paths.as_slice())
            .await?;

        self.copy_import_files(&instance_id, source_paths.as_slice(), &content_paths)
            .await?;

        self.pack_storage
            .update_pack_file_many(&instance_id, &content_paths, &pack_files)
            .await?;

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Edited,
                instance_id: instance_id.to_string(),
            })
            .await;

        Ok(())
    }
}
