//! Resolving a plugin-supplied relative path inside an instance directory.
//!
//! `write_instance_file` / `read_instance_file` are the single audited way into an instance, so
//! everything they are handed goes through here. Two separate checks are needed, and neither
//! replaces the other:
//!
//! * [`InstanceRelativePath`] (parsed in the domain) is pure string work. It refuses anything
//!   that is not a plain chain of name segments — absolute paths, drive letters, `..`, empty
//!   segments — before the filesystem is touched at all, so the functions here only ever receive
//!   a path that already passed it.
//! * [`resolve_instance_file_path`] then canonicalises what actually exists on disk and checks
//!   the result still sits under the instance root, which is what catches a **symlink** pointing
//!   out of the instance. A path can pass the first check and fail this one.
//!
//! The policy rules that need the filesystem or the plugin's identity live here as well:
//! [`check_plugin_write_allowed`] for `.metadata`, and the 8.3 alias check inside
//! [`resolve_instance_file_path`].

use std::path::{Path, PathBuf};

use crate::features::instance::domain::{
    ContentPath, InstanceError, InstanceId, InstanceRelativePath,
};
use crate::features::settings::{INSTANCE_METADATA_FOLDER_NAME, PLUGINS_FOLDER_NAME};

fn invalid(path: &str, reason: &str) -> InstanceError {
    InstanceError::InvalidRelativePath {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

fn escapes(path: &str, instance_id: &InstanceId) -> InstanceError {
    InstanceError::PathEscapesInstance {
        path: path.to_owned(),
        instance_id: instance_id.to_string(),
    }
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
    instance_id: &InstanceId,
    instance_dir: &Path,
    relative_path: &InstanceRelativePath,
) -> Result<PathBuf, InstanceError> {
    let relative = relative_path.to_path_buf();

    // An instance directory that is missing is not a directory we create here: the caller is
    // expected to have checked the instance exists.
    let canonical_root = crate::shared::io::infra::canonicalize(instance_dir).map_err(|err| {
        InstanceError::Storage(format!(
            "Failed to resolve directory of instance \"{instance_id}\": {err}"
        ))
    })?;

    let candidate = canonical_root.join(&relative);

    let existing =
        deepest_existing(&candidate).ok_or_else(|| escapes(relative_path.as_str(), instance_id))?;

    let canonical_existing = crate::shared::io::infra::canonicalize(existing)
        .map_err(|_| escapes(relative_path.as_str(), instance_id))?;

    if !canonical_existing.starts_with(&canonical_root) {
        return Err(escapes(relative_path.as_str(), instance_id));
    }

    if reaches_metadata_by_alias(relative_path.as_str(), &canonical_root, &canonical_existing) {
        return Err(invalid(
            relative_path.as_str(),
            &format!("path reaches `{INSTANCE_METADATA_FOLDER_NAME}` under another name"),
        ));
    }

    Ok(candidate)
}

/// Whether `relative_path` lands in `.metadata` on disk without naming it.
///
/// The `.metadata` rules ([`check_plugin_write_allowed`], [`ContentPath::parse`]) look at the
/// name a path spells, and a file system may know a directory by more than its name: on Windows,
/// `.metadata` also answers to its 8.3 short name (`METADA~1`). Which path *reaches* it is only
/// known once the path is canonicalised, so this compares the first component of the canonical
/// path with the first segment spelled. An alias is refused outright: there is no reason to use
/// one, and the rules for every spelling of the real name stay with the string checks.
fn reaches_metadata_by_alias(
    relative_path: &str,
    canonical_root: &Path,
    canonical_existing: &Path,
) -> bool {
    let is_metadata = |name: &str| name.eq_ignore_ascii_case(INSTANCE_METADATA_FOLDER_NAME);

    let spelled = relative_path
        .split(['/', '\\'])
        .next()
        .is_some_and(is_metadata);
    let reached = canonical_existing
        .strip_prefix(canonical_root)
        .ok()
        .and_then(|rest| rest.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .is_some_and(is_metadata);

    reached && !spelled
}

/// Refuse a plugin write into `.metadata`, except the plugin's own
/// `.metadata/plugins/<plugin_id>/`.
///
/// `.metadata` is the core's state (`instance.json` holds the install stage, the loader and
/// `pack_info`): a plugin that could rewrite it could tell the launcher an instance is something
/// it is not. Writes only: the instances directory is readable from the sandbox anyway.
///
/// `.metadata` is recognised case-insensitively (on Windows `.METADATA` is the same directory),
/// but permission is granted only for the exact spelling, and `plugin_id` is compared exactly:
/// a looser match would let one plugin into another's directory on a case-insensitive file system.
pub fn check_plugin_write_allowed(
    plugin_id: &str,
    relative_path: &InstanceRelativePath,
) -> Result<(), InstanceError> {
    // The segment rules already ran when the path was parsed, so `..` and the rest are gone.
    let mut segments = relative_path.segments();

    let Some(first) = segments.next() else {
        return Ok(());
    };

    if !first.eq_ignore_ascii_case(INSTANCE_METADATA_FOLDER_NAME) {
        return Ok(());
    }

    let own_subtree = first == INSTANCE_METADATA_FOLDER_NAME
        && segments.next() == Some(PLUGINS_FOLDER_NAME)
        && segments.next() == Some(plugin_id)
        // A path that stops at the directory itself is not a file in it.
        && segments.next().is_some();

    if own_subtree {
        return Ok(());
    }

    Err(invalid(
        relative_path.as_str(),
        &format!(
            "`{INSTANCE_METADATA_FOLDER_NAME}` belongs to the launcher; a plugin may only write \
             under `{INSTANCE_METADATA_FOLDER_NAME}/{PLUGINS_FOLDER_NAME}/{plugin_id}/`"
        ),
    ))
}

/// Check a content path — where an install puts a mod, a resource pack, a file of a plan — before
/// anything is downloaded for it.
///
/// The segment rules of [`InstanceRelativePath`], plus: nothing under `.metadata`. That is the
/// core's state (see [`check_plugin_write_allowed`]), and content is never part of it — not even
/// a plugin's own `.metadata/plugins/<id>/`, which a plugin writes through
/// `write_instance_file`, not through a plan. A provider's plan is the provider's word, so the
/// rule holds for built-in providers too. Spelled case-insensitively, as there.
///
/// Whether the path also stays inside the instance on disk (symlinks) is only known where the
/// file is written, by [`resolve_instance_file_path`].
pub fn check_content_path(relative_path: &str) -> Result<PathBuf, InstanceError> {
    let path = ContentPath::parse(relative_path)?;

    Ok(path.to_path_buf())
}

/// Refuse a payload over `limit` before it is written or read.
pub fn check_size(path: &str, size: u64, limit: u64) -> Result<(), InstanceError> {
    if size > limit {
        return Err(InstanceError::FileTooLarge {
            path: path.to_owned(),
            size,
            limit,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::reaches_metadata_by_alias;

    // Canonical paths as `canonicalize` would give them for an 8.3 alias of `.metadata`: the
    // short name itself never shows up in them.
    #[test]
    fn should_detect_a_short_name_of_metadata_when_the_path_spells_it_that_way() {
        assert!(reaches_metadata_by_alias(
            "METADA~1/instance.json",
            Path::new("/instances/a"),
            Path::new("/instances/a/.metadata/instance.json"),
        ));
        assert!(reaches_metadata_by_alias(
            "METADA~1/new/file",
            Path::new("/instances/a"),
            Path::new("/instances/a/.metadata"),
        ));
    }

    #[test]
    fn should_leave_metadata_to_the_string_checks_when_spelled_by_its_name() {
        for relative in [".metadata/instance.json", ".METADATA/instance.json"] {
            assert!(!reaches_metadata_by_alias(
                relative,
                Path::new("/instances/a"),
                Path::new("/instances/a/.metadata/instance.json"),
            ));
        }
    }

    #[test]
    fn should_not_flag_a_path_when_it_is_outside_metadata() {
        assert!(!reaches_metadata_by_alias(
            "MODS~1/sodium.jar",
            Path::new("/instances/a"),
            Path::new("/instances/a/mods/sodium.jar"),
        ));
        assert!(!reaches_metadata_by_alias(
            "mods/new.jar",
            Path::new("/instances/a"),
            Path::new("/instances/a"),
        ));
    }
}
