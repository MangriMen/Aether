use std::path::{Path, PathBuf};

use super::ManifestError;
use crate::shared::safe_path::{self, Separators};

/// A file inside a plugin's directory, as the manifest names it (`load.file`).
///
/// The only way to build one is [`TryFrom<PathBuf>`], which holds the path to the shared segment
/// rules ([`safe_path`]): relative, no empty, `.` or `..` segment, no drive letter, NUL or
/// reserved device name. The path is joined onto the plugin directory, so without this a manifest
/// could make the loader read (and hash) a file outside it. `/` and `\` both separate, so a
/// manifest names the same file on every platform.
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
        let refused = || ManifestError::InvalidLoadFile(path.display().to_string());

        let text = path.to_str().ok_or_else(refused)?;
        let normalized =
            safe_path::parse_relative(text, Separators::SlashOrBackslash).map_err(|reason| {
                tracing::debug!("Refusing `load.file` {text:?}: {reason}");
                refused()
            })?;

        Ok(Self(normalized.split('/').collect()))
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
    use rstest::rstest;

    fn parse(path: &str) -> Result<PluginFilePath, ManifestError> {
        PluginFilePath::try_from(PathBuf::from(path))
    }

    #[rstest]
    #[case::bare_file("plugin.wasm")]
    #[case::nested_file("bin/plugin.wasm")]
    fn should_accept_relative_file_inside_plugin_dir(#[case] file: &str) {
        assert!(parse(file).is_ok(), "{file} must be accepted");
    }

    #[rstest]
    #[case::parent_dir("../plugin.wasm")]
    #[case::nested_parent_dir("a/../../x.wasm")]
    #[case::absolute("/etc/passwd")]
    #[case::empty("")]
    fn should_reject_file_escaping_plugin_dir(#[case] file: &str) {
        assert!(
            matches!(parse(file), Err(ManifestError::InvalidLoadFile(_))),
            "{file} must be rejected"
        );
    }

    #[cfg(windows)]
    #[test]
    fn should_reject_drive_path() {
        assert!(matches!(
            parse("C:/x.wasm"),
            Err(ManifestError::InvalidLoadFile(_))
        ));
    }

    #[rstest]
    #[case::current_dir_prefix("./plugin.wasm")]
    #[case::repeated_separator("bin//plugin.wasm")]
    #[case::trailing_separator("plugin.wasm/")]
    #[case::backslash_traversal(r"bin\..\..\x.wasm")]
    #[case::backslash_root(r"\\server\share\x.wasm")]
    #[case::drive("C:/x.wasm")]
    #[case::device_name("nul.wasm")]
    #[case::trailing_dot("plugin.wasm.")]
    #[case::nul_byte("plugin\0.wasm")]
    fn should_reject_a_load_file_when_it_is_not_plain_names_on_any_platform(#[case] file: &str) {
        assert!(
            matches!(parse(file), Err(ManifestError::InvalidLoadFile(_))),
            "{file:?} must be rejected"
        );
    }

    #[test]
    fn should_read_a_backslash_as_a_separator_when_the_manifest_was_written_on_windows() {
        let file = parse(r"bin\plugin.wasm").unwrap();

        assert_eq!(
            file.as_path().components().collect::<Vec<_>>(),
            Path::new("bin")
                .join("plugin.wasm")
                .components()
                .collect::<Vec<_>>()
        );
    }
}
