use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use futures::TryStreamExt;
use path_slash::PathBufExt;

use crate::{
    features::{
        instance::{
            ContentFile, ContentMetadataService, ContentType, InstanceError, InstanceId, PackEntry,
            PackFile, PackStorage, app::ports::ListContentUseCasePort,
        },
        settings::LocationInfo,
    },
    shared::{hash::infra::sha1_async, io::infra::read_async},
};

pub struct ListContentUseCase {
    pack_storage: Arc<dyn PackStorage>,
    location_info: Arc<LocationInfo>,
    content_metadata: Arc<ContentMetadataService>,
}

impl ListContentUseCase {
    pub fn new(
        pack_storage: Arc<dyn PackStorage>,
        location_info: Arc<LocationInfo>,
        content_metadata: Arc<ContentMetadataService>,
    ) -> Self {
        Self {
            pack_storage,
            location_info,
            content_metadata,
        }
    }

    /// The instance's content files, each with its project's metadata as cached (see
    /// [`ContentMetadataService::attach`]): what is missing is fetched in the background, and
    /// the instance's `ContentSynced` event tells when to list again.
    pub async fn execute(
        &self,
        instance_id: InstanceId,
    ) -> Result<DashMap<String, ContentFile>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(&instance_id);

        let entries_by_path = Arc::new(self.get_entries_by_path(&instance_id).await?);

        let files = Arc::new(DashMap::new());
        for content_type in ContentType::iterator() {
            self.process_content_directory(
                &instance_id,
                &instance_dir,
                content_type,
                entries_by_path.clone(),
                files.clone(),
            )
            .await?;
        }

        let files = match Arc::try_unwrap(files) {
            Ok(map) => map,
            Err(arc) => (*arc).clone(),
        };
        self.content_metadata.attach(&instance_id, &files).await;

        Ok(files)
    }

    async fn get_entries_by_path(
        &self,
        instance_id: &InstanceId,
    ) -> Result<DashMap<String, PackEntry>, InstanceError> {
        let metadata = self.pack_storage.get_pack(instance_id).await?;

        Ok(metadata
            .files
            .into_iter()
            .map(|entry| (entry.file.clone(), entry))
            .collect())
    }

    async fn process_content_directory(
        &self,
        instance_id: &InstanceId,
        instance_dir: &Path,
        content_type: ContentType,
        entries_by_path: Arc<DashMap<String, PackEntry>>,
        files: Arc<DashMap<String, ContentFile>>,
    ) -> Result<(), InstanceError> {
        let content_dir = instance_dir.join(content_type.get_folder());

        // A folder the instance has not made yet holds no content. `try_exists` is `Ok(false)`
        // for it, not an error.
        if !tokio::fs::try_exists(&content_dir).await.unwrap_or(false) {
            return Ok(());
        }

        let read_dir = tokio::fs::read_dir(&content_dir)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        tokio_stream::wrappers::ReadDirStream::new(read_dir)
            .map_err(|e| InstanceError::Storage(e.to_string()))
            .try_for_each_concurrent(8, |entry| {
                let entries_ref = Arc::clone(&entries_by_path);
                let files_ref = Arc::clone(&files);

                async move {
                    let path = entry.path();

                    if !path.is_file() {
                        return Ok(());
                    }

                    if let Some(file) = self
                        .process_content_file(instance_id, &path, content_type, &entries_ref)
                        .await?
                    {
                        files_ref.insert(file.content_path.clone(), file);
                    }

                    Ok(())
                }
            })
            .await?;

        Ok(())
    }

    async fn process_content_file(
        &self,
        instance_id: &InstanceId,
        file_path: &Path,
        content_type: ContentType,
        entries_by_path: &DashMap<String, PackEntry>,
    ) -> Result<Option<ContentFile>, InstanceError> {
        let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) else {
            return Ok(None);
        };

        let file_size = tokio::fs::metadata(file_path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?
            .len();

        let is_disabled = file_name.ends_with(".disabled");

        let original_path = PathBuf::from(content_type.get_folder())
            .join(file_name)
            .to_slash_lossy()
            .to_string();

        let content_path = if is_disabled {
            original_path.trim_end_matches(".disabled").to_string()
        } else {
            original_path
        };

        let pack_file = self
            .get_or_create_pack_file(
                instance_id,
                file_path,
                file_name,
                &content_path,
                entries_by_path,
            )
            .await?;

        Ok(Some(ContentFile::from_pack_file(
            pack_file,
            content_path,
            content_type,
            file_size,
            is_disabled,
        )))
    }

    async fn get_or_create_pack_file(
        &self,
        instance_id: &InstanceId,
        file_path: &Path,
        file_name: &str,
        content_path: &str,
        entries_by_path: &DashMap<String, PackEntry>,
    ) -> Result<PackFile, InstanceError> {
        if let Some(entry) = entries_by_path.get(content_path) {
            return self
                .pack_storage
                .get_pack_file(instance_id, &entry.file)
                .await;
        }

        let file_content = read_async(file_path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        let hash = sha1_async(file_content)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        let pack_file = PackFile::from_hash(file_name.to_owned(), hash);

        self.pack_storage
            .update_pack_file(instance_id, content_path, &pack_file)
            .await?;

        Ok(pack_file)
    }
}

#[async_trait]
impl ListContentUseCasePort for ListContentUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
    ) -> Result<DashMap<String, ContentFile>, InstanceError> {
        self.execute(instance_id).await
    }
}
