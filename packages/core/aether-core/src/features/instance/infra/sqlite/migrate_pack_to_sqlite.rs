use crate::features::instance::{
    PackStorage,
    infra::{FsPackStorage, SqlitePackStorage},
};
use tracing::{info, warn};

pub async fn migrate_pack_to_sqlite(
    instance_id: &str,
    fs_storage: &FsPackStorage,
    sqlite_storage: &SqlitePackStorage,
) -> crate::Result<()> {
    let Ok(pack) = fs_storage.get_pack(instance_id).await else {
        return Ok(());
    };

    let existing_sqlite = sqlite_storage.get_pack(instance_id).await?;
    if !existing_sqlite.files.is_empty() {
        return Ok(());
    }

    info!(
        "Legacy pack found for instance {}. Starting migration...",
        instance_id
    );

    let mut paths = Vec::new();
    let mut files = Vec::new();

    for entry in pack.files {
        match fs_storage.get_pack_file(instance_id, &entry.file).await {
            Ok(pack_file) => {
                paths.push(entry.file);
                files.push(pack_file);
            }
            Err(e) => warn!(
                "Failed to read legacy file {} for {}: {}",
                entry.file, instance_id, e
            ),
        }
    }

    // Мигрируем всё одной транзакцией
    if !paths.is_empty() {
        sqlite_storage
            .update_pack_file_many(instance_id, &paths, &files)
            .await
            .map_err(|e| {
                warn!("Failed to migrate legacy pack: {}", e);
                e
            })?;
    }

    info!(
        "Migration for instance {} completed successfully.",
        instance_id
    );
    Ok(())
}
