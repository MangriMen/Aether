use std::path::PathBuf;

use crate::core::LauncherState;

pub async fn get_dir(instance_id: &str) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    Ok(state.location_info.instance_dir(instance_id))
}
