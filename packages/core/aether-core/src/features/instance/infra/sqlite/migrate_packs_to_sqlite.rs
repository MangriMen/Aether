use crate::features::instance::{
    InstanceStorage, PackStorage,
    infra::{FsPackStorage, SqlitePackStorage},
};
use tracing::{info, warn};

pub async fn migrate_packs_to_sqlite(
    instance_storage: &dyn InstanceStorage,
    fs_storage: &FsPackStorage,
    sqlite_storage: &SqlitePackStorage,
) -> crate::Result<()> {
    let instances = instance_storage.list().await?;

    for instance in instances {
        let instance_id = &instance.id;

        let Ok(pack) = fs_storage.get_pack(instance_id).await else {
            continue;
        };

        let existing_sqlite = sqlite_storage.get_pack(instance_id).await;
        if existing_sqlite.is_ok() {
            continue;
        }

        info!(
            "Legacy pack found for instance {}. Starting migration to SQLite...",
            instance_id
        );

        for entry in pack.files {
            let content_path = entry.file;
            match fs_storage.get_pack_file(instance_id, &content_path).await {
                Ok(pack_file) => {
                    if let Err(e) = sqlite_storage
                        .update_pack_file(instance_id, &content_path, &pack_file)
                        .await
                    {
                        warn!(
                            "Failed to migrate pack file {} for instance {}: {}",
                            content_path, instance_id, e
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to read legacy pack file {} for instance {}: {}",
                        content_path, instance_id, e
                    );
                }
            }
        }

        info!(
            "Migration of pack for instance {} to SQLite completed successfully.",
            instance_id
        );
    }

    Ok(())
}
