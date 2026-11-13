//! The segment rules every path from an untrusted source is held to before it is joined onto a
//! folder of the host: instance files, content paths, a plugin's own files, paths inside a
//! plugin's sandbox.
//!
//! The rules are applied to the raw string rather than through `std::path`. A plugin's path is
//! not this host's path: on Linux `mods\..\..\evil` is a single `Normal` component, so letting
//! the host's parser decide would make the very same input mean different things on different
//! platforms.
//!
//! The one rule that differs by platform is a `:` inside a name: on Windows it is a drive, a
//! prefix or a data stream, elsewhere an ordinary character of a file name the user may have.
//! A first segment spelled as a drive (`C:`) is refused everywhere, as it is absolute-looking.

use std::path::{Component, PathBuf};

/// Why [`parse_relative`] refused a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PathRejection {
    #[error("path is empty")]
    Empty,
    #[error("path contains a NUL byte")]
    NulByte,
    #[error("path contains `\\`; only `/` separates its segments")]
    Backslash,
    #[error("path is absolute")]
    Absolute,
    #[error("path has an empty segment (a repeated or trailing separator)")]
    EmptySegment,
    #[error("path contains a `.` or `..` segment")]
    DotSegment,
    #[error("path segment contains `:` (drive letter, UNC prefix or data stream)")]
    Colon,
    // Windows drops a trailing dot or space from a name, so `.metadata.` *is* `.metadata`
    // there: the segment would name another file than the one checked.
    #[error("path segment ends with a dot or a space")]
    TrailingDotOrSpace,
    #[error("path segment is a reserved Windows device name")]
    ReservedName,
    #[error("path does not consist of plain name segments")]
    NotPlain,
    // The rules below are not about the spelling of a path but about where it leads or what it
    // names; they are raised by the callers that know that, not by `parse_relative`.
    #[error("path leads out of the folder it belongs to")]
    EscapesRoot,
    #[error("path segment is not valid UTF-8")]
    NotUtf8,
    #[error("path is more than one segment")]
    NotSingleSegment,
    #[error("path is not a regular file")]
    NotRegularFile,
    #[error("path belongs to the launcher, not to content")]
    LauncherFolder,
}

/// What separates the segments of a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separators {
    /// `/` and `\` alike, on every host, so a path refused on Windows is refused on Linux too.
    /// For paths written by hand or by tools on any platform (manifests, packs).
    SlashOrBackslash,
    /// `/` only, and a `\` anywhere is refused rather than read as one or the other: for paths
    /// in a WASI sandbox, where `/` is the only separator and a `\` can only be a host-style
    /// spelling the host would split differently.
    SlashOnly,
}

/// Check `path` as a plain relative chain of name segments and return it `/`-joined.
///
/// Refused: an empty path, a NUL byte, a leading separator, empty, `.` or `..` segments, a
/// drive letter or a `:` that Windows reads as one ([`is_refused_colon`]), a segment ending with
/// a dot or a space, a reserved Windows device name, and under [`Separators::SlashOnly`] any `\`.
pub fn parse_relative(path: &str, separators: Separators) -> Result<String, PathRejection> {
    let normalized = parse_structure(path, separators, check_segment)?;

    // Belt and braces: whatever the host's own parser makes of the rebuilt path, it must still
    // be nothing but plain names. This is what would catch a form the rules above did not
    // anticipate on some platform.
    if !normalized
        .split('/')
        .collect::<PathBuf>()
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(PathRejection::NotPlain);
    }

    Ok(normalized)
}

/// [`parse_relative`] without the rules for names the host's file system treats specially (a
/// `:`, a trailing dot or space, a device name): only the structure of the path is checked.
///
/// For a name that is never joined onto a host folder but stands for one, such as the place a
/// folder of the user's is mounted in a sandbox: it is derived from the folder's real path, and
/// that folder may well be called `aux` or `My Pack ` on a file system that allows it.
pub fn parse_relative_structure(
    path: &str,
    separators: Separators,
) -> Result<String, PathRejection> {
    parse_structure(path, separators, check_structural_segment)
}

fn parse_structure(
    path: &str,
    separators: Separators,
    check: fn(usize, &str) -> Result<(), PathRejection>,
) -> Result<String, PathRejection> {
    if path.is_empty() {
        return Err(PathRejection::Empty);
    }

    if path.contains('\0') {
        return Err(PathRejection::NulByte);
    }

    if separators == Separators::SlashOnly && path.contains('\\') {
        return Err(PathRejection::Backslash);
    }

    if path.starts_with(['/', '\\']) {
        return Err(PathRejection::Absolute);
    }

    let mut normalized = String::with_capacity(path.len());

    for (index, segment) in path.split(['/', '\\']).enumerate() {
        check(index, segment)?;

        if index > 0 {
            normalized.push('/');
        }
        normalized.push_str(segment);
    }

    Ok(normalized)
}

fn check_structural_segment(_index: usize, segment: &str) -> Result<(), PathRejection> {
    match segment {
        "" => Err(PathRejection::EmptySegment),
        "." | ".." => Err(PathRejection::DotSegment),
        _ => Ok(()),
    }
}

/// The rules for the `index`th segment of a relative path.
fn check_segment(index: usize, segment: &str) -> Result<(), PathRejection> {
    check_structural_segment(index, segment)?;

    match segment {
        _ if is_refused_colon(index, segment) => Err(PathRejection::Colon),
        _ if segment.ends_with(['.', ' ']) => Err(PathRejection::TrailingDotOrSpace),
        _ if is_reserved_windows_name(segment) => Err(PathRejection::ReservedName),
        _ => Ok(()),
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
/// path must mean the same thing on every platform it is shared to.
pub fn is_reserved_windows_name(segment: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_a_backslash_when_only_slashes_separate() {
        for path in [r"a\b", r"..\x", r"cache\..\..\config", r"C:\Windows"] {
            assert_eq!(
                parse_relative(path, Separators::SlashOnly),
                Err(PathRejection::Backslash),
                "{path}"
            );
        }
    }

    #[test]
    fn should_read_a_backslash_as_a_separator_when_both_separate() {
        assert_eq!(
            parse_relative(r"bin\plugin.wasm", Separators::SlashOrBackslash),
            Ok("bin/plugin.wasm".to_owned())
        );
        assert_eq!(
            parse_relative(r"a\..\x", Separators::SlashOrBackslash),
            Err(PathRejection::DotSegment)
        );
    }

    #[test]
    fn should_refuse_each_unsafe_form_when_parsing_under_either_separator_rule() {
        for separators in [Separators::SlashOnly, Separators::SlashOrBackslash] {
            for (path, expected) in [
                ("", PathRejection::Empty),
                ("a\0b", PathRejection::NulByte),
                ("/etc/passwd", PathRejection::Absolute),
                ("a//b", PathRejection::EmptySegment),
                ("a/", PathRejection::EmptySegment),
                ("./a", PathRejection::DotSegment),
                ("a/../../b", PathRejection::DotSegment),
                ("C:/Windows", PathRejection::Colon),
                ("a/b.", PathRejection::TrailingDotOrSpace),
                ("a/b ", PathRejection::TrailingDotOrSpace),
                ("a/nul.txt", PathRejection::ReservedName),
                ("COM1", PathRejection::ReservedName),
            ] {
                assert_eq!(
                    parse_relative(path, separators),
                    Err(expected),
                    "{path:?} under {separators:?}"
                );
            }
        }
    }

    #[test]
    fn should_accept_names_the_host_treats_specially_when_only_the_structure_is_checked() {
        for path in [
            "home/u/aux",
            "Modpacks/aux.packs",
            "My Pack ",
            "x/com1.d",
            "a./b",
        ] {
            assert_eq!(
                parse_relative_structure(path, Separators::SlashOnly),
                Ok(path.to_owned()),
                "{path:?}"
            );
        }
    }

    #[test]
    fn should_still_refuse_an_unsafe_structure_when_only_the_structure_is_checked() {
        for (path, expected) in [
            ("", PathRejection::Empty),
            ("a\0b", PathRejection::NulByte),
            (r"a\b", PathRejection::Backslash),
            ("/a", PathRejection::Absolute),
            ("a//b", PathRejection::EmptySegment),
            ("a/./b", PathRejection::DotSegment),
            ("a/../b", PathRejection::DotSegment),
        ] {
            assert_eq!(
                parse_relative_structure(path, Separators::SlashOnly),
                Err(expected),
                "{path:?}"
            );
        }
    }

    #[test]
    fn should_refuse_a_data_stream_when_the_host_is_windows() {
        let result = parse_relative("a/file.txt:secret", Separators::SlashOnly);

        if cfg!(windows) {
            assert_eq!(result, Err(PathRejection::Colon));
        } else {
            assert_eq!(result, Ok("a/file.txt:secret".to_owned()));
        }
    }
}
