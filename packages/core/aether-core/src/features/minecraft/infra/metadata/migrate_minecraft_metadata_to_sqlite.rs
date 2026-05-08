use tracing::{info, warn};

use crate::{features::settings::LocationInfo, shared::remove_dir_all};

pub async fn migrate_minecraft_metadata_to_sqlite(location_info: &LocationInfo) {
    let minecraft_metadata_cache = location_info.cache_dir().join("minecraft");

    if minecraft_metadata_cache.exists() {
        match remove_dir_all(minecraft_metadata_cache).await {
            Ok(()) => {
                info!("Legacy Minecraft metadata found and removed");
            }
            Err(_) => {
                warn!("Failed to remove legacy Minecraft metadata");
            }
        }
    }
}
