use std::path::PathBuf;

use crate::{core::app::AetherContainer, features::settings::SettingsFeature};

pub async fn get_dir(instance_id: &str) -> crate::Result<PathBuf> {
    let container = AetherContainer::get();
    Ok(container.location_info().instance_dir(instance_id))
}
