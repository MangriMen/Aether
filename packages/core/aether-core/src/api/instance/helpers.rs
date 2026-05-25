use std::path::PathBuf;

use crate::core::LazyLocator;

pub async fn get_dir(instance_id: &str) -> crate::Result<PathBuf> {
    let locator = LazyLocator::get().await?;
    Ok(locator.location_info.instance_dir(instance_id))
}
