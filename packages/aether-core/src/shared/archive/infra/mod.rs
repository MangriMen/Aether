use std::path::Path;

use tracing::{info, warn};

use crate::shared::io::infra::create_dir_all;

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
        warn!(
            "Failed to get legacy {legacy_name} file name. Migration won't run again anyway because DB is now populated."
        );
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

/// Sets aside a legacy file that cannot be read, as `<migrated>/<name>.corrupt`.
///
/// Left in place it would fail the same way on every start; this keeps it for a manual fix.
pub async fn quarantine_unreadable_legacy_file(
    old_path: &Path,
    migrated_dir_name: &str,
    legacy_name: &str,
) {
    let Some(file_name) = old_path.file_name() else {
        return;
    };
    let migrated_path = old_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(migrated_dir_name);

    if let Err(err) = create_dir_all(&migrated_path).await {
        warn!("Failed to create migrated directory: {}", err);
        return;
    }

    let mut quarantined_name = file_name.to_os_string();
    quarantined_name.push(".corrupt");
    let quarantined_path = migrated_path.join(quarantined_name);

    match std::fs::rename(old_path, &quarantined_path) {
        Ok(()) => warn!("Unreadable legacy {legacy_name} file moved to {quarantined_path:?}"),
        Err(err) => warn!("Failed to set aside unreadable legacy {legacy_name} file: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_move_unreadable_file_to_migrated_with_corrupt_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("settings.json");
        std::fs::write(&old_path, "{ not json").unwrap();

        quarantine_unreadable_legacy_file(&old_path, "migrated", "settings").await;

        assert!(!old_path.exists());
        let kept = dir.path().join("migrated").join("settings.json.corrupt");
        assert_eq!(std::fs::read_to_string(kept).unwrap(), "{ not json");
    }
}
