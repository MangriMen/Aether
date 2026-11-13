//! Paths relative to an instance directory, validated once at the edge.
//!
//! [`InstanceRelativePath`] is pure string work. It refuses anything that is not a plain chain of
//! name segments — absolute paths, drive letters, `..`, empty segments — before the filesystem is
//! touched at all, so everything downstream receives a value that is already known to be sound.
//! Whether the path also stays inside the instance on disk (symlinks) is a different question,
//! answered where the file is reached.
//!
//! The segment rules themselves are shared with the other paths the core takes from outside
//! (see [`crate::shared::safe_path`]).

use std::fmt;
use std::path::PathBuf;

use super::InstanceError;
use crate::features::settings::INSTANCE_METADATA_FOLDER_NAME;
use crate::shared::safe_path::{self, PathRejection, Separators};

/// A path refused by [`InstanceRelativePath::parse`] or [`ContentPath::parse`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Path \"{path}\" is not a valid path inside an instance: {reason}")]
pub struct InvalidRelativePath {
    pub path: String,
    pub reason: PathRejection,
}

impl From<InvalidRelativePath> for InstanceError {
    fn from(err: InvalidRelativePath) -> Self {
        InstanceError::InvalidRelativePath {
            path: err.path,
            reason: err.reason,
        }
    }
}

fn invalid(path: &str, reason: PathRejection) -> InvalidRelativePath {
    InvalidRelativePath {
        path: path.to_owned(),
        reason,
    }
}

/// A plugin-supplied path checked to be a plain relative chain of name segments.
///
/// Both `/` and `\` are treated as separators no matter which host this runs on, so a path that
/// is refused on Windows is refused on Linux too — but for a `:` inside a name past a drive-like
/// first segment. The value is normalized: every `\` is a `/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceRelativePath(String);

impl InstanceRelativePath {
    /// Check `relative_path` against the segment rules and normalize it.
    pub fn parse(relative_path: &str) -> Result<Self, InvalidRelativePath> {
        safe_path::parse_relative(relative_path, Separators::SlashOrBackslash)
            .map(Self)
            .map_err(|rejection| invalid(relative_path, rejection))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path as the host spells it: the segments pushed one by one.
    pub fn to_path_buf(&self) -> PathBuf {
        self.segments().collect()
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    pub fn first_segment(&self) -> &str {
        self.segments().next().unwrap_or_default()
    }
}

impl fmt::Display for InstanceRelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A content path — where an install puts a mod, a resource pack, a file of a plan — checked
/// before anything is downloaded for it.
///
/// The segment rules of [`InstanceRelativePath`], plus: nothing under `.metadata`. That is the
/// core's state (a plugin may write only its own `.metadata/plugins/<id>/`), and content is never
/// part of it — not even that subtree, which a plugin writes through `write_instance_file`, not
/// through a plan. A provider's plan is the provider's word, so the rule holds for built-in
/// providers too. Spelled case-insensitively, as there.
///
/// Whether the path also stays inside the instance on disk (symlinks) is only known where the
/// file is written.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentPath(InstanceRelativePath);

impl ContentPath {
    pub fn parse(content_path: &str) -> Result<Self, InvalidRelativePath> {
        let path = InstanceRelativePath::parse(content_path)?;

        if path
            .first_segment()
            .eq_ignore_ascii_case(INSTANCE_METADATA_FOLDER_NAME)
        {
            return Err(invalid(content_path, PathRejection::LauncherFolder));
        }

        Ok(Self(path))
    }

    pub fn as_relative(&self) -> &InstanceRelativePath {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.0.to_path_buf()
    }

    /// The last segment: the name of the file itself.
    pub fn file_name(&self) -> &str {
        self.0.as_str().rsplit('/').next().unwrap_or_default()
    }

    pub fn into_string(self) -> String {
        self.0.0
    }

    /// The path as the file system tells paths apart: without case on Windows and macOS, exactly
    /// elsewhere. Two paths with the same key are the same file there.
    pub fn file_system_key(&self) -> String {
        self.key(cfg!(any(windows, target_os = "macos")))
    }

    /// The path in lower case, for comparing names whatever the file system does.
    pub fn case_insensitive_key(&self) -> String {
        self.as_str().to_lowercase()
    }

    /// [`Self::file_system_key`] for a file system that does or does not ignore case.
    pub fn key(&self, ignores_case: bool) -> String {
        if ignores_case {
            self.as_str().to_lowercase()
        } else {
            self.as_str().to_owned()
        }
    }

    /// This path with `suffix` appended to the last segment (`mods/a.jar` + `.disabled`),
    /// checked against the segment rules again.
    pub fn with_suffix(&self, suffix: &str) -> Result<InstanceRelativePath, InvalidRelativePath> {
        InstanceRelativePath::parse(&format!("{}{suffix}", self.as_str()))
    }
}

impl fmt::Display for ContentPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ContentPath> for String {
    fn from(path: ContentPath) -> Self {
        path.into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_reserved_windows_device_names() {
        for path in [
            "CON",
            "mods/nul",
            "mods/NUL.txt",
            "aux.tar.gz",
            "Prn",
            "com1",
            "mods/LPT9.jar",
            "COM\u{b9}",
            "lpt\u{b2}.txt",
            "CONIN$",
            "conout$",
            "con:stream",
            "nul\\x",
        ] {
            assert!(InstanceRelativePath::parse(path).is_err(), "{path}");
        }
    }

    #[test]
    fn should_accept_names_that_only_resemble_device_names() {
        for path in [
            "console.txt",
            "mods/null.jar",
            "com0",
            "com10.jar",
            "lpt",
            "mods/auxiliary",
            "a.con",
            "my.nul",
        ] {
            assert!(InstanceRelativePath::parse(path).is_ok(), "{path}");
        }
    }

    #[test]
    fn should_refuse_reserved_names_in_content_paths_too() {
        assert!(ContentPath::parse("mods/aux.jar").is_err());
    }

    #[test]
    fn should_normalize_backslashes_when_parsing() {
        let path = InstanceRelativePath::parse(r"config\mod\settings.json").unwrap();

        assert_eq!(path.as_str(), "config/mod/settings.json");
        assert_eq!(path.first_segment(), "config");
        assert_eq!(path.segments().count(), 3);
    }

    #[test]
    fn should_refuse_metadata_in_any_case_when_parsing_a_content_path() {
        for path in [
            ".metadata/x",
            ".METADATA/x",
            r".Metadata\plugins\a\b",
            ".metadata",
        ] {
            assert!(
                ContentPath::parse(path).is_err(),
                "`{path}` must be refused"
            );
        }
    }

    #[test]
    fn should_expose_the_file_name_when_parsing_a_content_path() {
        let path = ContentPath::parse("mods/sodium.jar").unwrap();

        assert_eq!(path.file_name(), "sodium.jar");
        assert_eq!(path.as_str(), "mods/sodium.jar");
    }

    #[test]
    fn should_key_a_content_path_by_case_only_when_the_file_system_ignores_it() {
        let path = ContentPath::parse("Mods/Sodium.jar").unwrap();

        assert_eq!(path.key(true), "mods/sodium.jar");
        assert_eq!(path.key(false), "Mods/Sodium.jar");
        assert_eq!(path.case_insensitive_key(), "mods/sodium.jar");
        assert_eq!(path.clone().into_string(), "Mods/Sodium.jar");
    }

    #[test]
    fn should_allow_metadata_as_a_later_segment_when_parsing_a_content_path() {
        assert!(ContentPath::parse("mods/.metadata/a.jar").is_ok());
    }
}
