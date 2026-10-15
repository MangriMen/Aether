use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tracing::warn;

use crate::{
    features::{
        instance::{Instance, InstanceError, InstanceStorage, infra::fs::instance::InstanceV2},
        settings::LocationInfo,
    },
    shared::io::infra::{read_json_async, remove_dir_all, write_json_async},
};

/// An id that cannot name a directory directly under `instances/` is no instance at all.
fn check_plain_id(id: &str) -> Result<(), InstanceError> {
    if LocationInfo::is_plain_instance_id(id) {
        Ok(())
    } else {
        Err(InstanceError::NotFound {
            instance_id: id.to_owned(),
        })
    }
}

pub struct FsInstanceStorage {
    location_info: Arc<LocationInfo>,
}

impl FsInstanceStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    async fn load_instance_internal(&self, path: &Path) -> Result<Instance, InstanceError> {
        match read_json_async::<InstanceV2>(path).await {
            Ok(instance) => Ok(instance.into()),
            Err(original_err) => match read_json_async::<super::instance::InstanceV1>(path).await {
                Ok(v1) => {
                    warn!("Migrating instance from V1 at {:?}", path);
                    let migrated: Instance = v1.into();
                    let v2 = InstanceV2::from(&migrated);

                    if let Err(e) = write_json_async(path, &v2).await {
                        warn!("Failed to save migrated instance: {:?}", e);
                    }

                    Ok(migrated)
                }
                Err(_) => Err(InstanceError::Storage(original_err.to_string())),
            },
        }
    }
}

#[async_trait]
impl InstanceStorage for FsInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        let instances_dir = self.location_info.instances_dir();

        if !instances_dir.exists() {
            return Ok(Vec::default());
        }

        let Ok(entries) = instances_dir.read_dir() else {
            return Ok(Vec::default());
        };

        let mut instances = Vec::new();

        for entry in entries.flatten() {
            let instance_json_path = self
                .location_info
                .instance_metadata_file_with_instance_dir(&entry.path());

            match self.load_instance_internal(&instance_json_path).await {
                Ok(instance) => instances.push(instance),
                Err(err) => {
                    warn!(
                        "Failed to read instance at {:?}: {:?}",
                        instance_json_path, err
                    );
                }
            }
        }

        return Ok(instances);
    }

    async fn get(&self, id: &str) -> Result<Instance, InstanceError> {
        // Otherwise an `instance.json` planted anywhere in the config directory, e.g. under
        // `plugins/<id>/`, would pass as an instance and let its id be used for writes.
        check_plain_id(id)?;
        let path = self.location_info.instance_metadata_file(id);
        self.load_instance_internal(&path).await
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let path = self.location_info.instance_metadata_file(instance.id());
        write_json_async(&path, InstanceV2::from(instance))
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        Ok(())
    }

    async fn remove(&self, id: &str) -> Result<(), InstanceError> {
        check_plain_id(id)?;
        let path = self.location_info.instance_dir(id);
        remove_dir_all(&path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::features::{instance::InstanceBuilder, minecraft::ModLoader};

    /// An instance stored as usual, then moved under `plugins/`, where a plugin could have put a
    /// copy of it.
    async fn setup_planted() -> (TempDir, FsInstanceStorage, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp_dir.path().join("settings"),
            temp_dir.path().to_path_buf(),
        ));
        let storage = FsInstanceStorage::new(location_info.clone());

        let instance = InstanceBuilder::new(
            "planted".to_owned(),
            "Planted".to_owned(),
            "1.20.1".to_owned(),
            ModLoader::Vanilla,
        )
        .build();
        storage.upsert(&instance).await.unwrap();

        let planted = location_info.plugin_dir("planted");
        std::fs::create_dir_all(location_info.plugins_dir()).unwrap();
        std::fs::rename(location_info.instance_dir("planted"), &planted).unwrap();

        (temp_dir, storage, planted)
    }

    #[tokio::test]
    async fn should_not_find_an_instance_when_its_id_leaves_the_instances_directory() {
        let (_temp_dir, storage, planted) = setup_planted().await;
        let absolute = planted.to_string_lossy().into_owned();

        for id in [
            "../plugins/planted",
            "..\\plugins\\planted",
            absolute.as_str(),
        ] {
            let err = storage.get(id).await.expect_err(id);
            assert!(matches!(err, InstanceError::NotFound { .. }), "{id}: {err}");
        }
    }

    #[tokio::test]
    async fn should_not_remove_a_directory_when_the_id_leaves_the_instances_directory() {
        let (_temp_dir, storage, planted) = setup_planted().await;
        let absolute = planted.to_string_lossy().into_owned();

        for id in ["../plugins/planted", absolute.as_str()] {
            storage.remove(id).await.expect_err(id);
        }

        assert!(
            planted.exists(),
            "a refused id must not have removed anything"
        );
    }
}
