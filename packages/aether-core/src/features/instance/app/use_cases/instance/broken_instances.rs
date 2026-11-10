use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::{
    BrokenInstanceSource, InstanceStorage, LegacyInstanceSource, ListBrokenInstancesUseCasePort,
    RemoveBrokenInstanceUseCasePort, RemoveInstanceUseCasePort,
};
use crate::features::instance::{
    BrokenInstance, BrokenInstanceOrigin, InstanceError, InstanceId, folder_key,
};
use crate::features::settings::LocationInfo;

/// The instances that are stored but cannot be read, so that the user can see them (and why)
/// instead of them being missing from the list.
pub struct ListBrokenInstancesUseCase {
    storage: Arc<dyn InstanceStorage>,
    records: Arc<dyn BrokenInstanceSource>,
    legacy: Arc<dyn LegacyInstanceSource>,
    location_info: Arc<LocationInfo>,
}

impl ListBrokenInstancesUseCase {
    pub fn new(
        storage: Arc<dyn InstanceStorage>,
        records: Arc<dyn BrokenInstanceSource>,
        legacy: Arc<dyn LegacyInstanceSource>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            storage,
            records,
            legacy,
            location_info,
        }
    }
}

#[async_trait]
impl ListBrokenInstancesUseCasePort for ListBrokenInstancesUseCase {
    async fn execute(&self) -> Result<Vec<BrokenInstance>, InstanceError> {
        let mut found: Vec<BrokenInstance> = self
            .records
            .list_broken()
            .await?
            .into_iter()
            .map(|record| BrokenInstance {
                // The folder is worked out from the id, never taken from outside: an id that
                // names no folder has none.
                dir: InstanceId::try_from(record.id.clone())
                    .ok()
                    .map(|id| self.location_info.instance_dir(&id)),
                id: record.id,
                origin: BrokenInstanceOrigin::Database,
                reason: record.reason,
            })
            .collect();

        // The files of a folder that has a record are not looked at: the record is what is read.
        let stored = self.storage.stored_ids().await?;
        found.extend(self.legacy.list_unmigrated(&stored).await);

        found.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(found)
    }
}

/// Removes an instance that cannot be read, by the id the list of broken ones gave.
///
/// Only an instance that is broken now: a stale list must not remove one that has been repaired
/// since.
pub struct RemoveBrokenInstanceUseCase {
    storage: Arc<dyn InstanceStorage>,
    broken: Arc<dyn ListBrokenInstancesUseCasePort>,
    records: Arc<dyn BrokenInstanceSource>,
    remove_instance: Arc<dyn RemoveInstanceUseCasePort>,
}

impl RemoveBrokenInstanceUseCase {
    pub fn new(
        storage: Arc<dyn InstanceStorage>,
        broken: Arc<dyn ListBrokenInstancesUseCasePort>,
        records: Arc<dyn BrokenInstanceSource>,
        remove_instance: Arc<dyn RemoveInstanceUseCasePort>,
    ) -> Self {
        Self {
            storage,
            broken,
            records,
            remove_instance,
        }
    }
}

#[async_trait]
impl RemoveBrokenInstanceUseCasePort for RemoveBrokenInstanceUseCase {
    async fn execute(&self, id: String) -> Result<(), InstanceError> {
        let target = self
            .broken
            .execute()
            .await?
            .into_iter()
            .find(|broken| broken.id == id)
            .ok_or_else(|| InstanceError::NotBroken {
                instance_id: id.clone(),
            })?;

        let stored = self.storage.stored_ids().await?;
        match InstanceId::try_from(target.id) {
            // Another record that names the same folder owns it as much: only this record goes.
            Ok(instance_id) if shares_its_folder(&instance_id, &stored) => {
                self.records.remove_record(&id).await
            }
            // The usual removal: the lock, the file watch, the folder, the pack data, the record.
            Ok(instance_id) => self.remove_instance.execute(instance_id).await,
            // No folder can be named by such an id: only its record goes, by the text it has.
            Err(_) => self.records.remove_record(&id).await,
        }
    }
}

/// Whether a record other than `instance_id` itself names the same folder.
fn shares_its_folder(instance_id: &InstanceId, stored: &[String]) -> bool {
    let key = folder_key(instance_id.as_str());
    stored
        .iter()
        .filter(|other| other.as_str() != instance_id.as_str())
        .any(|other| folder_key(other) == key)
}
