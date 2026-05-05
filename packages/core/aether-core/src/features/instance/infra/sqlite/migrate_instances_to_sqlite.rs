use crate::features::instance::{
    InstanceStorage,
    infra::{FsInstanceStorage, SqliteInstanceStorage},
};
use tracing::{info, warn};

pub async fn migrate_instances_to_sqlite(
    fs_storage: &FsInstanceStorage,
    sqlite_storage: &SqliteInstanceStorage,
) -> crate::Result<()> {
    let old_instances = match fs_storage.list().await {
        Ok(instances) if !instances.is_empty() => instances,
        _ => return Ok(()),
    };

    let existing_sqlite = sqlite_storage.list().await?;
    if !existing_sqlite.is_empty() {
        return Ok(());
    }

    info!(
        "Legacy instances found (count: {}). Starting migration to SQLite...",
        old_instances.len()
    );

    // 3. Переносим данные
    for instance in old_instances {
        let id = instance.id.clone();
        if let Err(e) = sqlite_storage.upsert(&instance).await {
            warn!("Failed to migrate instance {}: {}", id, e);
        }
    }

    info!("Migration of instances to SQLite completed successfully.");

    // 4. По поводу удаления старых файлов:
    // В отличие от credentials, здесь файлы instance.json в папках можно не удалять сразу,
    // чтобы у пользователя была возможность "откатиться", просто удалив sqlite.db.
    // Если же ты хочешь пометить их как мигрированные, можно создать пустой файл-флаг .migrated
    // Но обычно в таких проектах файлы оставляют как бэкап.

    Ok(())
}
