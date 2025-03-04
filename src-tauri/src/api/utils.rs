use std::path::Path;

use crate::AetherLauncherResult;

#[tauri::command]
pub fn reveal_in_explorer(path: String, exact: bool) -> AetherLauncherResult<()> {
    crate::utils::file::reveal_in_explorer(Path::new(&path), exact)
}
