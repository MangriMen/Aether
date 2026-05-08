use crate::{
    features::java::{
        JavaInstallationService, JavaStorage, app::JavaApplicationError, infra::SqliteJavaStorage,
    },
    shared::rename,
};
use std::path::Path;
use tracing::{info, warn};

pub async fn migrate_java_to_sqlite(
    java_dir: &Path,
    installation_service: &dyn JavaInstallationService,
    sqlite_storage: &SqliteJavaStorage,
) -> crate::Result<()> {
    let existing_java = sqlite_storage
        .list()
        .await
        .map_err(JavaApplicationError::from)?;

    if !existing_java.is_empty() {
        return Ok(());
    }

    info!(
        "Java storage is empty. Starting discovery in {:?}...",
        java_dir
    );

    // Instead of json migration run scanning
    let discovered_java = match installation_service.discover_installations(java_dir).await {
        Ok(list) if !list.is_empty() => list,
        Ok(_) => {
            info!("No Java installations found in {:?}", java_dir);
            return Ok(());
        }
        Err(e) => {
            warn!(
                "Failed to discover Java installations during migration: {}",
                e
            );
            return Ok(());
        }
    };

    info!(
        "Found {} Java installations. Migrating to SQLite...",
        discovered_java.len()
    );

    for java in discovered_java {
        let version_name = java.version().to_string();
        let major = java.major_version();

        if let Err(e) = sqlite_storage.upsert(java).await {
            warn!(
                "Failed to save Java {} (major: {}) to SQLite: {}",
                version_name, major, e
            );
        }
    }

    info!("Java discovery and migration to SQLite completed successfully.");

    let legacy_json = java_dir.join("java_versions.json");
    if legacy_json.exists() {
        match rename(&legacy_json, &java_dir.join("java_versions.back.json")).await {
            Ok(()) => info!("Legacy java_versions.json removed."),
            Err(e) => warn!("Failed to remove legacy java_versions.json: {}", e),
        }
    }

    Ok(())
}
