//! Resolving a plugin-supplied relative path inside an instance directory (T-1.3).
//!
//! `write_instance_file` / `read_instance_file` are the single audited way into an instance, so
//! everything they are handed goes through here first. Two separate checks are needed, and
//! neither replaces the other:
//!
//! * [`validate_relative_path`] is pure string work. It refuses anything that is not a plain
//!   chain of name segments — absolute paths, drive letters, `..`, empty segments — before the
//!   filesystem is touched at all.
//! * [`resolve_instance_file_path`] then canonicalises what actually exists on disk and checks
//!   the result still sits under the instance root, which is what catches a **symlink** pointing
//!   out of the instance. A path can pass the first check and fail this one.
//!
//! Note that the segment rules are applied to the raw string rather than through `std::path`.
//! A plugin's path is not this host's path: on Linux `mods\..\..\evil` is a single `Normal`
//! component, so letting the host's parser decide would make the very same input mean different
//! things on different platforms.

use std::path::{Component, Path, PathBuf};

use crate::features::instance::domain::InstanceError;

/// Largest file a plugin may write into, or read out of, an instance in one call.
///
/// The bytes travel through the plugin's own linear memory, which defaults to 256 MiB (Q8), so
/// a much larger cap would buy nothing: the plugin could not hold the file anyway. Content that
/// genuinely is large belongs in `download_to_cache` (T-1.2), which never materialises the body
/// inside the sandbox.
pub const MAX_INSTANCE_FILE_BYTES: u64 = 64 * 1024 * 1024;

fn invalid(path: &str, reason: &str) -> InstanceError {
    InstanceError::InvalidRelativePath {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

fn escapes(path: &str, instance_id: &str) -> InstanceError {
    InstanceError::PathEscapesInstance {
        path: path.to_owned(),
        instance_id: instance_id.to_owned(),
    }
}

/// Check a plugin-supplied path is a plain relative chain of name segments, and rebuild it.
///
/// Both `/` and `\` are treated as separators no matter which host this runs on, so a path that
/// is refused on Windows is refused on Linux too.
pub fn validate_relative_path(relative_path: &str) -> Result<PathBuf, InstanceError> {
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

    for segment in relative_path.split(['/', '\\']) {
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
            // A drive letter (`C:`), a UNC/verbatim prefix, or a Windows alternate data stream
            // (`file.txt:secret`) all show up as a `:` inside a segment.
            _ if segment.contains(':') => {
                return Err(invalid(
                    relative_path,
                    "path segment contains `:` (drive letter, UNC prefix or data stream)",
                ));
            }
            _ => result.push(segment),
        }
    }

    // Belt and braces: whatever the host's own parser makes of the rebuilt path, it must still
    // be nothing but plain names. This is what would catch a form the rules above did not
    // anticipate on some platform.
    if !result
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(invalid(
            relative_path,
            "path does not consist of plain name segments",
        ));
    }

    Ok(result)
}

/// The deepest ancestor of `path` (possibly `path` itself) that exists on disk.
///
/// `symlink_metadata` on purpose: a symlink counts as existing even when its target does not, so
/// a dangling link is handed to `canonicalize` below and refused there rather than silently
/// treated as a free path.
fn deepest_existing(path: &Path) -> Option<&Path> {
    let mut current = path;
    loop {
        if current.symlink_metadata().is_ok() {
            return Some(current);
        }
        current = current.parent()?;
    }
}

/// Resolve `relative_path` against the directory of `instance_id` and prove the result stays
/// inside it.
///
/// The returned path is the one to actually use. The target itself need not exist — a write
/// creates it — but every part of it that *does* exist is canonicalised, so neither an
/// intermediate directory symlink nor a symlinked target can point out of the instance.
pub fn resolve_instance_file_path(
    instance_id: &str,
    instance_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, InstanceError> {
    let relative = validate_relative_path(relative_path)?;

    // An instance directory that is missing is not a directory we create here: the caller is
    // expected to have checked the instance exists.
    let canonical_root = crate::shared::io::infra::canonicalize(instance_dir).map_err(|err| {
        InstanceError::Storage(format!(
            "Failed to resolve directory of instance \"{instance_id}\": {err}"
        ))
    })?;

    let candidate = canonical_root.join(&relative);

    let existing =
        deepest_existing(&candidate).ok_or_else(|| escapes(relative_path, instance_id))?;

    let canonical_existing = crate::shared::io::infra::canonicalize(existing)
        .map_err(|_| escapes(relative_path, instance_id))?;

    if !canonical_existing.starts_with(&canonical_root) {
        return Err(escapes(relative_path, instance_id));
    }

    Ok(candidate)
}

/// Refuse a payload over [`MAX_INSTANCE_FILE_BYTES`] before it is written or read.
pub fn check_size(path: &str, size: u64) -> Result<(), InstanceError> {
    if size > MAX_INSTANCE_FILE_BYTES {
        return Err(InstanceError::FileTooLarge {
            path: path.to_owned(),
            size,
            limit: MAX_INSTANCE_FILE_BYTES,
        });
    }

    Ok(())
}
