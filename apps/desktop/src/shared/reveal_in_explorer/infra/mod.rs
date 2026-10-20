use std::{io, path::Path};

/// Refuses anything that is not an existing path inside one of the roots: a URL or a path
/// to an executable elsewhere would be opened by the shell otherwise.
pub fn ensure_within_roots(path: &Path, roots: &[&Path]) -> io::Result<()> {
    let canonical = path.canonicalize()?;

    let allowed = roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| canonical.starts_with(root));

    if allowed {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "The path is outside the launcher folders",
        ))
    }
}

pub fn reveal_in_explorer(path: &Path, exact: bool) -> crate::Result<()> {
    if exact {
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
    fn should_allow_a_path_inside_a_root() {
        let root = tempfile::tempdir().unwrap();
        let inner = root.path().join("instances");
        std::fs::create_dir(&inner).unwrap();

        assert!(ensure_within_roots(&inner, &[root.path()]).is_ok());
    }

    #[test]
    fn should_refuse_a_path_outside_the_roots() {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();

        let err = ensure_within_roots(other.path(), &[root.path()]).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn should_refuse_a_url() {
        let root = tempfile::tempdir().unwrap();

        assert!(ensure_within_roots(Path::new("https://example.com"), &[root.path()]).is_err());
    }

    #[test]
    fn should_refuse_a_dot_dot_escape() {
        let root = tempfile::tempdir().unwrap();
        let inner = root.path().join("a");
        std::fs::create_dir(&inner).unwrap();
        let escape = inner.join("..").join("..");

        assert!(ensure_within_roots(&escape, &[&inner]).is_err());
    }
}
