use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::{
        LocationInfo, StorageKind, StorageUsage, StorageUsageEntry,
        app::ports::GetStorageUsageUseCasePort,
    },
    shared::housekeeping::dir_size,
};

/// How much disk the launcher's folders take, by what they hold.
pub struct GetStorageUsageUseCase {
    location_info: Arc<LocationInfo>,
}

impl GetStorageUsageUseCase {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

#[async_trait]
impl GetStorageUsageUseCasePort for GetStorageUsageUseCase {
    async fn execute(&self) -> StorageUsage {
        let info = &self.location_info;
        // The folders do not overlap, so they are measured side by side.
        let (assets, libraries, versions, natives, java, instances) = (
            info.assets_dir(),
            info.libraries_dir(),
            info.versions_dir(),
            info.natives_dir(),
            info.java_dir(),
            info.instances_dir(),
        );
        let (assets, libraries, versions, natives, java, instances) = tokio::join!(
            dir_size(&assets),
            dir_size(&libraries),
            dir_size(&versions),
            dir_size(&natives),
            dir_size(&java),
            dir_size(&instances),
        );

        StorageUsage {
            entries: vec![
                StorageUsageEntry {
                    kind: StorageKind::MinecraftAssets,
                    bytes: assets,
                },
                StorageUsageEntry {
                    kind: StorageKind::MinecraftLibraries,
                    bytes: libraries,
                },
                StorageUsageEntry {
                    kind: StorageKind::MinecraftVersions,
                    bytes: versions + natives,
                },
                StorageUsageEntry {
                    kind: StorageKind::Java,
                    bytes: java,
                },
                StorageUsageEntry {
                    kind: StorageKind::Instances,
                    bytes: instances,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_measure_each_folder_by_what_it_holds_when_asked_for_usage() {
        let temp = tempfile::tempdir().unwrap();
        let info = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().join("config"),
        ));
        for (dir, size) in [
            (info.assets_dir(), 4),
            (info.libraries_dir(), 3),
            (info.versions_dir(), 2),
            (info.natives_dir(), 1),
            (info.java_dir(), 7),
            (info.instance_dir("one"), 9),
        ] {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("file"), vec![0; size]).unwrap();
        }

        let usage = GetStorageUsageUseCase::new(info).execute().await;

        let bytes: Vec<_> = usage.entries.iter().map(|entry| entry.bytes).collect();
        assert_eq!(bytes, [4, 3, 3, 7, 9]);
    }
}
