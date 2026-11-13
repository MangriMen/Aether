use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use dashmap::DashMap;
use futures::TryStreamExt;
use path_slash::PathBufExt;

use crate::{
    features::{
        instance::{
            ContentFile, ContentFileView, ContentMetadataService, ContentPath, ContentType,
            InstanceError, InstanceId, InstanceLocks, PackFile, PackStorage,
            app::{DISABLED_SUFFIX, ports::ListContentUseCasePort},
        },
        settings::LocationInfo,
    },
    shared::hash::infra::sha1_file_hash_async,
};

pub struct ListContentUseCase {
    pack_storage: Arc<dyn PackStorage>,
    location_info: Arc<LocationInfo>,
    content_metadata: Arc<ContentMetadataService>,
    instance_locks: Arc<InstanceLocks>,
}

/// What the listing read once, before it walked the folders.
struct Snapshot {
    /// Every path with a record, usable or not.
    recorded: HashSet<ContentPath>,
    /// The records that can be used.
    records: HashMap<ContentPath, PackFile>,
}

/// How a listed file stands with the storage.
enum Record {
    Stored,
    /// Nothing is recorded for it.
    Missing,
    /// A record exists but its hash cannot be used.
    Unusable,
}

/// A file of a content folder, with the record it is listed by.
struct Found {
    pack_file: PackFile,
    content_path: ContentPath,
    content_type: ContentType,
    file_path: PathBuf,
    size: u64,
    modified: Option<std::time::SystemTime>,
    is_disabled: bool,
    record: Record,
}

impl Found {
    fn into_content_file(self) -> ContentFile {
        ContentFile::from_pack_file(
            self.pack_file,
            self.content_path,
            self.content_type,
            self.size,
            self.is_disabled,
        )
    }
}

impl ListContentUseCase {
    pub fn new(
        pack_storage: Arc<dyn PackStorage>,
        location_info: Arc<LocationInfo>,
        content_metadata: Arc<ContentMetadataService>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            pack_storage,
            location_info,
            content_metadata,
            instance_locks,
        }
    }

    /// The instance's content files, each with its project's metadata as cached (see
    /// [`ContentMetadataService::attach`]): what is missing is fetched in the background, and
    /// the instance's `ContentSynced` event tells when to list again.
    pub async fn execute(
        &self,
        instance_id: InstanceId,
    ) -> Result<DashMap<String, ContentFileView>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(&instance_id);

        let snapshot = Arc::new(self.read_snapshot(&instance_id).await?);

        let found = Arc::new(Mutex::new(Vec::new()));
        for content_type in ContentType::iterator() {
            self.process_content_directory(
                &instance_id,
                &instance_dir,
                content_type,
                snapshot.clone(),
                found.clone(),
            )
            .await?;
        }

        let mut found = std::mem::take(
            &mut *found
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        self.record_found(&instance_id, &mut found).await?;

        let views: DashMap<String, ContentFileView> = found
            .into_iter()
            .map(|found| {
                let file = found.into_content_file();
                (file.content_path.clone().into_string(), file.into())
            })
            .collect();
        self.content_metadata.attach(&instance_id, &views).await;

        Ok(views)
    }

    async fn read_snapshot(&self, instance_id: &InstanceId) -> Result<Snapshot, InstanceError> {
        let recorded = self
            .pack_storage
            .get_pack(instance_id)
            .await?
            .files
            .into_iter()
            .map(|entry| entry.file)
            .collect();
        let records = self.pack_storage.get_pack_files(instance_id).await?;

        Ok(Snapshot { recorded, records })
    }

    async fn process_content_directory(
        &self,
        instance_id: &InstanceId,
        instance_dir: &Path,
        content_type: ContentType,
        snapshot: Arc<Snapshot>,
        found: Arc<Mutex<Vec<Found>>>,
    ) -> Result<(), InstanceError> {
        let content_dir = instance_dir.join(content_type.get_folder());

        // A folder the instance has not made yet holds no content. `try_exists` is `Ok(false)`
        // for it, not an error.
        if !tokio::fs::try_exists(&content_dir).await.unwrap_or(false) {
            return Ok(());
        }

        let read_dir = tokio::fs::read_dir(&content_dir)
            .await
            .map_err(InstanceError::storage)?;

        tokio_stream::wrappers::ReadDirStream::new(read_dir)
            .map_err(InstanceError::storage)
            .try_for_each_concurrent(8, |entry| {
                let snapshot_ref = Arc::clone(&snapshot);
                let found_ref = Arc::clone(&found);

                async move {
                    let path = entry.path();

                    if !path.is_file() {
                        return Ok(());
                    }

                    // `x` next to `x.disabled`: the enabled file is the one that loads, so it is
                    // the one listed. Skipping the copy up front keeps it from being hashed.
                    if let Some(enabled) = path
                        .to_str()
                        .and_then(|path| path.strip_suffix(DISABLED_SUFFIX))
                        && Path::new(enabled).is_file()
                    {
                        return Ok(());
                    }

                    if let Some(file) = self
                        .process_content_file(instance_id, &path, content_type, &snapshot_ref)
                        .await?
                    {
                        found_ref
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push(file);
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
        snapshot: &Snapshot,
    ) -> Result<Option<Found>, InstanceError> {
        let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) else {
            return Ok(None);
        };

        // A file a change removes while the listing runs is left out, not an error.
        let metadata = match tokio::fs::metadata(file_path).await {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(InstanceError::storage(err)),
        };
        let file_size = metadata.len();
        let modified = metadata.modified().ok();

        let original_path = PathBuf::from(content_type.get_folder())
            .join(file_name)
            .to_slash_lossy()
            .to_string();

        // Only one suffix: `a.jar.disabled.disabled` is the disabled `a.jar.disabled`.
        let (content_path, is_disabled) = match original_path.strip_suffix(DISABLED_SUFFIX) {
            Some(enabled) => (enabled, true),
            None => (original_path.as_str(), false),
        };

        // A file whose name can never be a content path (a trailing dot or space, a `:` where
        // the platform refuses it, a `\` that parsing would turn into a separator) cannot be
        // tracked or changed: it is left out of the list.
        let content_path = match ContentPath::parse(content_path) {
            Ok(parsed) if parsed.as_str() == content_path => parsed,
            Ok(_) => {
                tracing::warn!(
                    "Skipping \"{original_path}\" of \"{instance_id}\": its name is not a single path segment"
                );
                return Ok(None);
            }
            Err(err) => {
                tracing::warn!(
                    "Skipping \"{original_path}\" of \"{instance_id}\": not a valid content path: {err}"
                );
                return Ok(None);
            }
        };

        let (pack_file, record) = if let Some(pack_file) = snapshot.records.get(&content_path) {
            (pack_file.clone(), Record::Stored)
        } else {
            let hash = match sha1_file_hash_async(file_path).await {
                Ok(hash) => hash,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(err) => return Err(InstanceError::storage(err)),
            };
            let record = if snapshot.recorded.contains(&content_path) {
                Record::Unusable
            } else {
                Record::Missing
            };
            (PackFile::from_hash(file_name.to_owned(), hash), record)
        };

        Ok(Some(Found {
            pack_file,
            content_path,
            content_type,
            file_path: file_path.to_path_buf(),
            size: file_size,
            modified,
            is_disabled,
            record,
        }))
    }

    /// Records the files the storage knows nothing usable about, all under one hold of the
    /// instance's lock. Not while a plan or another change holds it: the listing shows the files
    /// as found and records them on a later pass, rather than waiting on a lock that can be held
    /// for a whole install.
    async fn record_found(
        &self,
        instance_id: &InstanceId,
        found: &mut [Found],
    ) -> Result<(), InstanceError> {
        if found
            .iter()
            .all(|file| matches!(file.record, Record::Stored))
        {
            return Ok(());
        }
        let Ok(_lock) = self.instance_locks.try_lock(instance_id) else {
            return Ok(());
        };

        for file in found
            .iter_mut()
            .filter(|file| !matches!(file.record, Record::Stored))
        {
            // The hashing took a while: a change that held the lock meanwhile may have removed
            // or replaced the file, and a record of what is no longer there would stay.
            let unchanged = tokio::fs::metadata(&file.file_path)
                .await
                .is_ok_and(|metadata| {
                    metadata.len() == file.size && metadata.modified().ok() == file.modified
                });
            if !unchanged {
                continue;
            }

            file.pack_file = self.record_file(instance_id, file).await?;
        }

        Ok(())
    }

    async fn record_file(
        &self,
        instance_id: &InstanceId,
        file: &Found,
    ) -> Result<PackFile, InstanceError> {
        if matches!(file.record, Record::Unusable) {
            // A plan may have replaced the record since the snapshot.
            if let Some(stored) = self
                .pack_storage
                .find_pack_file(instance_id, &file.content_path)
                .await?
            {
                return Ok(stored);
            }
            // What is there keeps everything but its hash.
            if let Some(repaired) = self
                .pack_storage
                .rehash_pack_file(instance_id, &file.content_path, &file.pack_file.hash)
                .await?
            {
                return Ok(repaired);
            }
        }

        // A plan may have recorded the file (with its owner) since the snapshot, and that record
        // is not ours to replace.
        self.pack_storage
            .insert_pack_file_if_absent(instance_id, &file.content_path, &file.pack_file)
            .await
    }
}

#[async_trait]
impl ListContentUseCasePort for ListContentUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
    ) -> Result<DashMap<String, ContentFileView>, InstanceError> {
        self.execute(instance_id).await
    }
}
