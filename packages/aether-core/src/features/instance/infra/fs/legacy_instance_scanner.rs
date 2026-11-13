use std::{collections::HashSet, sync::Arc};

use tracing::debug;

use crate::{
    features::{
        instance::{
            BrokenInstance, BrokenInstanceOrigin, BrokenInstanceReason, Instance, InstanceError,
            InstanceId, folder_key, infra::fs::instance::InstanceV2,
        },
        settings::LocationInfo,
    },
    shared::io::infra::read_json_async,
};

/// Finds the legacy `instance.json` files that did not make it into the database.
///
/// Only a folder that still has the file counts: the file of a migrated instance is set aside,
/// and an instance made since has none. A folder whose name has a record in the database is left
/// alone, its file or not, because the record is what the launcher reads. Nothing is written:
/// this only looks.
pub struct LegacyInstanceScanner {
    location_info: Arc<LocationInfo>,
}

impl LegacyInstanceScanner {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    /// `stored_ids`: the ids of every record in the database. Compared without regard to case,
    /// as the file system does, so `Foo` and `foo` are one folder.
    pub async fn scan(&self, stored_ids: &[String]) -> Vec<BrokenInstance> {
        let stored: HashSet<String> = stored_ids.iter().map(|id| folder_key(id)).collect();

        let Ok(mut entries) = tokio::fs::read_dir(self.location_info.instances_dir()).await else {
            return Vec::new();
        };

        let mut found = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            // A name that is not a valid id names no instance (the watcher skips it too).
            let Some(id) = entry
                .file_name()
                .to_str()
                .and_then(|name| InstanceId::try_from(name.to_owned()).ok())
            else {
                continue;
            };
            if stored.contains(&folder_key(id.as_str())) {
                continue;
            }
            let file = self.location_info.instance_metadata_file(&id);
            if !file.is_file() {
                continue;
            }

            let reason = match Self::read(&file).await {
                Ok(file_instance) if file_instance.id() != &id => {
                    BrokenInstanceReason::LegacyIdMismatch {
                        file_id: file_instance.id().to_string(),
                    }
                }
                Ok(_) => BrokenInstanceReason::NotMigrated,
                Err(reason) => reason,
            };
            debug!("Instance {id} has a legacy file the launcher does not use: {reason}");
            found.push(BrokenInstance {
                dir: Some(self.location_info.instance_dir(&id)),
                id: id.to_string(),
                origin: BrokenInstanceOrigin::LegacyFile,
                reason,
            });
        }
        found.sort_by(|a, b| a.id.cmp(&b.id));
        found
    }

    async fn read(file: &std::path::Path) -> Result<Instance, BrokenInstanceReason> {
        let read = match read_json_async::<InstanceV2>(file).await {
            Ok(v2) => Instance::try_from(v2),
            Err(v2_err) => match read_json_async::<super::instance::InstanceV1>(file).await {
                Ok(v1) => Instance::try_from(v1),
                Err(_) => Err(InstanceError::storage(v2_err)),
            },
        };
        read.map_err(|err| match err {
            InstanceError::InvalidGameVersion { value, .. } => {
                BrokenInstanceReason::InvalidGameVersion { value }
            }
            other => BrokenInstanceReason::UnreadableFile {
                details: other.to_string(),
            },
        })
    }
}

#[async_trait::async_trait]
impl crate::features::instance::LegacyInstanceSource for LegacyInstanceScanner {
    async fn list_unmigrated(&self, stored_ids: &[String]) -> Vec<BrokenInstance> {
        self.scan(stored_ids).await
    }
}
