use std::path::Path;

pub fn reveal_in_explorer(path: &Path, exact: bool) -> crate::Result<()> {
    if exact {
        Ok(open::that(path)?)
    } else {
        #[cfg(target_os = "windows")]
        {
            let reveal_path = format!("/select,{}", path.display().to_string().replace("/", "\\"));
            Ok(open::with_detached(reveal_path, "explorer")?)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let parent_folder = match path.is_file() {
                true => path.parent().unwrap().to_path_buf(),
                false => path.to_path_buf(),
            };

            Ok(open::that(parent_folder)?)
        }
    }
}
