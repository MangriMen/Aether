use std::{io, path::PathBuf};

pub fn reveal_in_explorer(path: &PathBuf, exact: bool) -> io::Result<()> {
    if exact {
        open::that(&path)
    } else {
        #[cfg(target_os = "windows")]
        {
            let reveal_path = format!("/select,{}", path.display().to_string().replace("/", "\\"));
            open::with_detached(reveal_path, "explorer")
        }

        #[cfg(not(target_os = "windows"))]
        {
            let parent_folder = match path.is_file() {
                true => path.parent().unwrap().to_path_buf(),
                false => path.to_path_buf(),
            };

            open::that(parent_folder)
        }
    }
}
