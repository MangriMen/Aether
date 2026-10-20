use std::{io, path::Path};

pub fn reveal_in_explorer(path: &Path, exact: bool) -> crate::Result<()> {
    if exact {
        // `open` runs a file through the shell, so only a folder may be opened this way.
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Only a folder can be opened",
            )
            .into());
        }

        Ok(open::that(path)?)
    } else {
        #[cfg(target_os = "windows")]
        {
            let reveal_path = format!("/select,{}", path.display().to_string().replace('/', "\\"));
            Ok(open::with_detached(reveal_path, "explorer")?)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let parent_folder = if path.is_file() {
                path.parent().unwrap().to_path_buf()
            } else {
                path.to_path_buf()
            };

            Ok(open::that(parent_folder)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_to_open_a_file_exactly() {
        let file = std::env::temp_dir().join(format!("aether-reveal-{}.bat", std::process::id()));
        std::fs::write(&file, b"").unwrap();

        let result = reveal_in_explorer(&file, true);
        std::fs::remove_file(&file).unwrap();

        assert!(result.is_err());
    }
}
