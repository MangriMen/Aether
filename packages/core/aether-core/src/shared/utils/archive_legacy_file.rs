use std::path::Path;

use tracing::{info, warn};

use crate::shared::create_dir_all;

pub async fn archive_legacy_file(old_path: &Path, migrated_dir_name: &str, legacy_name: &str) {
    let migrated_path = old_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(migrated_dir_name);

    if let Err(err) = create_dir_all(&migrated_path).await {
        warn!("Failed to create migrated directory: {}", err);
        return;
    }

    let Some(file_name) = old_path.file_name() else {
        warn!("
                Failed to get legacy {legacy_name} file name. Migration won't run again anyway because DB is now populated.");
        return;
    };

    let backup_path = migrated_path.join(file_name);

    if let Err(e) = std::fs::rename(old_path, &backup_path) {
        warn!(
            "Failed to rename legacy {legacy_name} file: {}. Migration won't run again anyway because DB is now populated.",
            e
        );
    } else {
        info!("Legacy {legacy_name} moved to {:?}", backup_path);
    }
}
