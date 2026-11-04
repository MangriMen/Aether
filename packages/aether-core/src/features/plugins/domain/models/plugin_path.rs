//! Paths inside a plugin's WASI sandbox, checked once where a plugin (or a settings file) hands
//! one over.

use std::fmt;

use crate::shared::safe_path::{self, PathRejection, Separators};

/// Why a string is not a [`PluginPath`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PluginPathRejection {
    #[error("a plugin path must start with `/`")]
    NotAbsolute,
    #[error("a plugin path must not be `/` itself")]
    Root,
    #[error(transparent)]
    Segment(#[from] PathRejection),
}

/// A string refused by [`PluginPath::parse`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Plugin path \"{path}\" is not valid: {reason}")]
pub struct InvalidPluginPath {
    pub path: String,
    pub reason: PluginPathRejection,
}

/// An absolute path in a plugin's sandbox, as the plugin sees it: `/cache/pack.zip`,
/// `/instances/<id>`, `/mnt/d/packs`.
///
/// `/` followed by one or more name segments under the rules of [`safe_path`] with `/` as the
/// only separator: no empty, `.` or `..` segment, no `\`, drive letter, NUL, reserved device
/// name, or trailing dot or space. Two such paths compare segment by segment, and the segments
/// below a mount can be joined onto its host folder without leaving it by name. Whether the
/// plugin may have the path, and where it leads on the host, is decided by whoever maps it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginPath(String);

impl PluginPath {
    pub fn parse(path: &str) -> Result<Self, InvalidPluginPath> {
        let invalid = |reason| InvalidPluginPath {
            path: path.to_owned(),
            reason,
        };

        let Some(relative) = path.strip_prefix('/') else {
            return Err(invalid(PluginPathRejection::NotAbsolute));
        };

        if relative.is_empty() {
            return Err(invalid(PluginPathRejection::Root));
        }

        // The shared rules would call the rest of `//x` absolute; here it is an empty segment.
        if relative.starts_with('/') {
            return Err(invalid(PathRejection::EmptySegment.into()));
        }

        safe_path::parse_relative(relative, Separators::SlashOnly)
            .map(|normalized| Self(format!("/{normalized}")))
            .map_err(|rejection| invalid(rejection.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The name segments, without the leading `/`.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0[1..].split('/')
    }

    /// The segments of this path below `base`; `None` unless this path is `base` or lies under
    /// it by whole segments (`/cachex` is not under `/cache`).
    pub fn strip_prefix<'a>(
        &'a self,
        base: &PluginPath,
    ) -> Option<impl Iterator<Item = &'a str> + use<'a>> {
        let rest = self.0.strip_prefix(&base.0)?;
        if !(rest.is_empty() || rest.starts_with('/')) {
            return None;
        }

        // `rest` is empty or `/a/b`: splitting yields a leading empty piece either way.
        Some(rest.split('/').skip(1))
    }

    /// Whether this path is `base` or lies under it.
    pub fn is_within(&self, base: &PluginPath) -> bool {
        self.strip_prefix(base).is_some()
    }
}

impl fmt::Display for PluginPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<PluginPath> for String {
    fn from(path: PluginPath) -> Self {
        path.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn rejection(path: &str) -> PluginPathRejection {
        PluginPath::parse(path)
            .expect_err(&format!("{path:?} must be refused"))
            .reason
    }

    #[rstest]
    #[case::mount_root("/cache")]
    #[case::file_in_mount("/cache/downloads/sha1/abc")]
    #[case::instance("/instances/my-pack/pack.toml")]
    #[case::drive_folder("/mnt/d/packs")]
    #[case::dotted_names("/cache/.hidden/a.b.c")]
    fn should_accept_a_plain_absolute_path_when_parsing(#[case] path: &str) {
        let parsed = PluginPath::parse(path).unwrap();

        assert_eq!(parsed.as_str(), path);
    }

    #[rstest]
    #[case::empty("")]
    #[case::relative("cache/x")]
    #[case::unc(r"\\server\share")]
    #[case::drive("C:/Windows")]
    #[case::drive_backslash(r"C:\Windows")]
    fn should_refuse_a_path_when_it_does_not_start_with_a_slash(#[case] path: &str) {
        assert_eq!(rejection(path), PluginPathRejection::NotAbsolute);
    }

    #[test]
    fn should_refuse_the_root_when_parsing() {
        assert_eq!(rejection("/"), PluginPathRejection::Root);
    }

    #[rstest]
    #[case::double_leading_slash("//cache", PathRejection::EmptySegment)]
    #[case::repeated_slash("/cache//x", PathRejection::EmptySegment)]
    #[case::trailing_slash("/instances/id/", PathRejection::EmptySegment)]
    #[case::current_dir("/cache/./x", PathRejection::DotSegment)]
    #[case::parent_dir("/cache/../config", PathRejection::DotSegment)]
    #[case::backslash_separator(r"/cache\..\config", PathRejection::Backslash)]
    #[case::backslash_in_name(r"/cache/a\b", PathRejection::Backslash)]
    #[case::unc_inside(r"/cache/\\server\share", PathRejection::Backslash)]
    #[case::nul_byte("/cache/a\0b", PathRejection::NulByte)]
    #[case::drive_segment("/C:/Windows", PathRejection::Colon)]
    #[case::device_name("/cache/nul", PathRejection::ReservedName)]
    #[case::device_name_with_extension("/cache/COM1.txt", PathRejection::ReservedName)]
    #[case::trailing_dot("/cache/x.", PathRejection::TrailingDotOrSpace)]
    #[case::trailing_space("/cache/x ", PathRejection::TrailingDotOrSpace)]
    fn should_refuse_a_segment_when_it_is_not_a_plain_name(
        #[case] path: &str,
        #[case] expected: PathRejection,
    ) {
        assert_eq!(rejection(path), PluginPathRejection::Segment(expected));
    }

    #[test]
    fn should_refuse_a_data_stream_when_the_host_is_windows() {
        let result = PluginPath::parse("/cache/pack.zip:stream");

        if cfg!(windows) {
            assert_eq!(
                result.unwrap_err().reason,
                PluginPathRejection::Segment(PathRejection::Colon)
            );
        } else {
            assert!(result.is_ok(), "a `:` is part of a name off Windows");
        }
    }

    #[test]
    fn should_give_the_segments_below_a_base_when_the_path_lies_under_it() {
        let cache = PluginPath::parse("/cache").unwrap();
        let file = PluginPath::parse("/cache/sub/pack.zip").unwrap();

        assert_eq!(
            file.strip_prefix(&cache).unwrap().collect::<Vec<_>>(),
            ["sub", "pack.zip"]
        );
        assert_eq!(cache.strip_prefix(&cache).unwrap().count(), 0);
        assert_eq!(
            file.segments().collect::<Vec<_>>(),
            ["cache", "sub", "pack.zip"]
        );
    }

    #[test]
    fn should_not_be_within_a_base_when_it_only_shares_a_prefix_of_a_name() {
        let cache = PluginPath::parse("/cache").unwrap();

        assert!(!PluginPath::parse("/cachex/a").unwrap().is_within(&cache));
        assert!(!PluginPath::parse("/Cache/a").unwrap().is_within(&cache));
        assert!(!cache.is_within(&PluginPath::parse("/cache/a").unwrap()));
    }
}
