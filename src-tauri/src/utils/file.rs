use std::path::Path;

use super::{AetherLauncherError, AetherLauncherResult};

pub fn reveal_in_explorer(path: &Path, exact: bool) -> AetherLauncherResult<()> {
    if exact {
        open::that(path).map_err(|err| AetherLauncherError {
            message: err.to_string(),
        })
    } else {
        #[cfg(target_os = "windows")]
        {
            let reveal_path = format!("/select,{}", path.display().to_string().replace("/", "\\"));
            open::with_detached(reveal_path, "explorer").map_err(|err| AetherLauncherError {
                message: err.to_string(),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            let parent_folder = match path.is_file() {
                true => path.parent().unwrap().to_path_buf(),
                false => path.to_path_buf(),
            };

            open::that(parent_folder).map_err(|err| AetherLauncherError {
                message: err.to_string(),
            })
        }
    }
}
