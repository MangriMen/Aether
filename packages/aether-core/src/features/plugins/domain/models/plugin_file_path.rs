use std::path::{Component, Path, PathBuf};

use super::ManifestError;

/// A file inside a plugin's directory, as the manifest names it (`load.file`).
///
/// The only way to build one is [`TryFrom<PathBuf>`], which refuses an empty path, an absolute
/// one and any `..`. The path is joined onto the plugin directory, so without this a manifest
/// could make the loader read (and hash) a file outside it.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct PluginFilePath(PathBuf);

impl PluginFilePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<PathBuf> for PluginFilePath {
    type Error = ManifestError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let stays_inside = !path.as_os_str().is_empty()
            && path
                .components()
                .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));

        if stays_inside {
            Ok(Self(path))
        } else {
            Err(ManifestError::InvalidLoadFile(path.display().to_string()))
        }
    }
}

impl TryFrom<&str> for PluginFilePath {
    type Error = ManifestError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Self::try_from(PathBuf::from(path))
    }
}

impl From<PluginFilePath> for PathBuf {
    fn from(path: PluginFilePath) -> Self {
        path.0
    }
}

impl AsRef<Path> for PluginFilePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(path: &str) -> Result<PluginFilePath, ManifestError> {
        PluginFilePath::try_from(PathBuf::from(path))
    }

    #[test]
    fn should_accept_relative_file_inside_plugin_dir() {
        assert!(parse("plugin.wasm").is_ok());
        assert!(parse("bin/plugin.wasm").is_ok());
        assert!(parse("./plugin.wasm").is_ok());
    }

    #[test]
    fn should_reject_file_escaping_plugin_dir() {
        for file in ["../plugin.wasm", "a/../../x.wasm", "/etc/passwd", ""] {
            assert!(
                matches!(parse(file), Err(ManifestError::InvalidLoadFile(_))),
                "{file} must be rejected"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn should_reject_drive_path() {
        assert!(matches!(
            parse("C:/x.wasm"),
            Err(ManifestError::InvalidLoadFile(_))
        ));
    }
}
