//! Paths relative to an instance directory, validated once at the edge.
//!
//! [`InstanceRelativePath`] is pure string work. It refuses anything that is not a plain chain of
//! name segments — absolute paths, drive letters, `..`, empty segments — before the filesystem is
//! touched at all, so everything downstream receives a value that is already known to be sound.
//! Whether the path also stays inside the instance on disk (symlinks) is a different question,
//! answered where the file is reached.
//!
//! Note that the segment rules are applied to the raw string rather than through `std::path`.
//! A plugin's path is not this host's path: on Linux `mods\..\..\evil` is a single `Normal`
//! component, so letting the host's parser decide would make the very same input mean different
//! things on different platforms.
//!
//! The one rule that differs by platform is a `:` inside a name: on Windows it is a drive, a
//! prefix or a data stream, elsewhere an ordinary character of a file name the user may have.
//! A first segment spelled as a drive (`C:`) is refused everywhere, as it is absolute-looking.

use std::fmt;
use std::path::{Component, PathBuf};

use super::InstanceError;
use crate::features::settings::INSTANCE_METADATA_FOLDER_NAME;

/// A path refused by [`InstanceRelativePath::parse`] or [`ContentPath::parse`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Path \"{path}\" is not a valid path inside an instance: {reason}")]
pub struct InvalidRelativePath {
    pub path: String,
    pub reason: String,
}

impl From<InvalidRelativePath> for InstanceError {
    fn from(err: InvalidRelativePath) -> Self {
        InstanceError::InvalidRelativePath {
            path: err.path,
            reason: err.reason,
        }
    }
}

fn invalid(path: &str, reason: &str) -> InvalidRelativePath {
    InvalidRelativePath {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

/// A plugin-supplied path checked to be a plain relative chain of name segments.
///
/// Both `/` and `\` are treated as separators no matter which host this runs on, so a path that
/// is refused on Windows is refused on Linux too — but for a `:` inside a name past a drive-like
/// first segment ([`is_refused_colon`]). The value is normalized: every `\` is a `/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceRelativePath(String);

impl InstanceRelativePath {
    /// Check `relative_path` against the segment rules and normalize it.
    pub fn parse(relative_path: &str) -> Result<Self, InvalidRelativePath> {
        if relative_path.is_empty() {
            return Err(invalid(relative_path, "path is empty"));
        }

        if relative_path.contains('\0') {
            return Err(invalid(relative_path, "path contains a NUL byte"));
        }

        if relative_path.starts_with('/') || relative_path.starts_with('\\') {
            return Err(invalid(
                relative_path,
                "path is absolute; only paths relative to the instance root are accepted",
            ));
        }

        let mut result = PathBuf::new();
        let mut normalized = String::with_capacity(relative_path.len());

        for (index, segment) in relative_path.split(['/', '\\']).enumerate() {
            match segment {
                "" => {
                    return Err(invalid(
                        relative_path,
                        "path has an empty segment (a repeated or trailing separator)",
                    ));
                }
                "." | ".." => {
                    return Err(invalid(
                        relative_path,
                        "path contains a `.` or `..` segment",
                    ));
                }
                _ if is_refused_colon(index, segment) => {
                    return Err(invalid(
                        relative_path,
                        "path segment contains `:` (drive letter, UNC prefix or data stream)",
                    ));
                }
                // Windows drops a trailing dot or space from a name, so `.metadata.` *is*
                // `.metadata` there: the segment would name another file than the one checked.
                _ if segment.ends_with(['.', ' ']) => {
                    return Err(invalid(
                        relative_path,
                        "path segment ends with a dot or a space",
                    ));
                }
                _ if is_reserved_windows_name(segment) => {
                    return Err(invalid(
                        relative_path,
                        "path segment is a reserved Windows device name",
                    ));
                }
                _ => {
                    result.push(segment);
                    if index > 0 {
                        normalized.push('/');
                    }
                    normalized.push_str(segment);
                }
            }
        }

        // Belt and braces: whatever the host's own parser makes of the rebuilt path, it must
        // still be nothing but plain names. This is what would catch a form the rules above did
        // not anticipate on some platform.
        if !result
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(invalid(
                relative_path,
                "path does not consist of plain name segments",
            ));
        }

        Ok(Self(normalized))
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

/// Whether a `:` in the `index`th segment refuses the path.
///
/// On Windows a drive letter (`C:`), a UNC/verbatim prefix, or an alternate data stream
/// (`file.txt:secret`) all show up as a `:` inside a segment, so any `:` is refused. Elsewhere it
/// is part of a name (`mods/a:b.jar` is a file a Linux user may have), and only a first segment
/// spelled as a drive is refused, so a path absolute on Windows is refused on every platform.
fn is_refused_colon(index: usize, segment: &str) -> bool {
    if cfg!(windows) {
        return segment.contains(':');
    }

    let bytes = segment.as_bytes();
    index == 0 && bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Whether `segment` names a Windows device (`CON`, `NUL`, `COM1`, ...).
///
/// Windows resolves such a name to the device whatever the case and whatever follows the first dot
/// or colon (`nul.txt`, `COM1.tar.gz`, `aux:stream`), so a file with it cannot be created or read
/// as a file there. Refused on every host for the same reason `\` is a separator everywhere: a
/// pack must mean the same thing on every platform it is shared to.
fn is_reserved_windows_name(segment: &str) -> bool {
    let stem = segment.split(['.', ':']).next().unwrap_or_default();
    let stem = stem.trim_end_matches(' ').to_ascii_uppercase();

    match stem.as_str() {
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" => true,
        _ => {
            let numbered = |prefix: &str| {
                stem.strip_prefix(prefix).is_some_and(|rest| {
                    let mut chars = rest.chars();
                    matches!(
                        (chars.next(), chars.next()),
                        (Some('1'..='9' | '\u{b9}' | '\u{b2}' | '\u{b3}'), None)
                    )
                })
            };
            numbered("COM") || numbered("LPT")
        }
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
            return Err(invalid(
                content_path,
                &format!(
                    "`{INSTANCE_METADATA_FOLDER_NAME}` belongs to the launcher, not to content"
                ),
            ));
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
