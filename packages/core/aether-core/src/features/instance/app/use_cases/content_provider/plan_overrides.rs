//! Reading the overrides of an install plan and copying them out into staging; writing them into
//! the instance is the executor's, through the same checks as the planned files.
//!
//! Paths are the source's own, `/`-separated, never parsed as host paths, so an entry means the
//! same file on every platform.
//!
//! Security: an archive with any entry escaping its root (`../evil`, an absolute path) is refused
//! as a whole. Symlinks are never materialised, as they could point out of the instance or the
//! source. A directory source is read from a handle on its root, one component at a time and
//! never through a symlink ([`open_in`]), so a directory swapped for a symlink after listing does
//! not lead out of it. What an archive unpacks to is bounded over all sources of a plan together
//! (`ContentInstallLimits`): files when listed, bytes as staged, so a lying header does not help.

use std::{
    fs::File,
    io::{Read, copy},
    path::{Path, PathBuf},
};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::{ambient_authority, fs::Dir};

use crate::features::instance::{InstanceError, OverrideLocation};

/// One file of the overrides, by the path it gets in the instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OverrideFile {
    pub relative_path: String,
    /// What the source says the file holds: an archive entry's header, a file's length when it
    /// was listed. Good for refusing early, never for the limit itself.
    pub declared_size: u64,
    source: OverrideFileSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OverrideFileSource {
    /// The index of the entry in the archive.
    ArchiveEntry(usize),
    /// A file of a directory, at its `relative_path` under the directory.
    DirectoryFile,
}

/// An override copied out into the staging directory.
pub(super) struct StagedOverride {
    pub relative_path: String,
    pub path: PathBuf,
}

/// The files `source` holds, in the order of the source (sorted for a directory), but no more
/// than `max_files + 1`: listing stops there, and more than `max_files` means the source is over
/// the limit.
pub(super) async fn list_overrides(
    source: &OverrideLocation,
    max_files: usize,
) -> Result<Vec<OverrideFile>, InstanceError> {
    let source = source.clone();

    blocking(move || match &source {
        OverrideLocation::Archive {
            archive_path,
            subdir,
        } => list_archive(archive_path, subdir, max_files),
        OverrideLocation::Directory { path } => list_directory(path, max_files),
    })
    .await
}

/// Copy the `files` of each source into `staging`, each under a name of its own, and refuse them
/// all once more than `max_bytes` are copied out of the sources together. What was written by
/// then is left in `staging`, removed with the rest of the plan's.
pub(super) async fn stage_overrides(
    sources: Vec<(OverrideLocation, Vec<OverrideFile>)>,
    staging: &Path,
    max_bytes: u64,
) -> Result<Vec<StagedOverride>, InstanceError> {
    let staging = staging.to_owned();

    blocking(move || {
        std::fs::create_dir_all(&staging).map_err(|err| storage(&err))?;

        let mut staged = Vec::new();
        let mut copied: u64 = 0;

        for (source, files) in sources {
            let (mut archive, root) = match &source {
                OverrideLocation::Archive { archive_path, .. } => {
                    (Some(open_archive(archive_path)?), None)
                }
                OverrideLocation::Directory { path } => (None, Some(open_root(path)?)),
            };

            for file in files {
                let path = staging.join(format!("override-{}", staged.len()));
                // One byte over what is left tells a file that fits from one that does not.
                let remaining = max_bytes.saturating_sub(copied);

                let written = match (&file.source, archive.as_mut(), &root) {
                    (OverrideFileSource::ArchiveEntry(entry), Some(archive), _) => {
                        let mut entry = archive
                            .by_index(*entry)
                            .map_err(|err| invalid_archive(&err))?;
                        let mut out = File::create(&path).map_err(|err| storage(&err))?;
                        copy(
                            &mut (&mut entry).take(remaining.saturating_add(1)),
                            &mut out,
                        )
                    }
                    (OverrideFileSource::DirectoryFile, _, Some(root)) => {
                        // Reached again from the root: the directory may have changed since it
                        // was listed, and no symlink put in since is followed out of it.
                        let input = open_in(root, &file.relative_path).map_err(|err| {
                            InstanceError::Storage(format!(
                                "Override \"{}\" is no longer a file of its directory: {err}",
                                file.relative_path
                            ))
                        })?;
                        // Counted as it is read: the file may have grown since it was listed.
                        let mut out = File::create(&path).map_err(|err| storage(&err))?;
                        copy(&mut input.take(remaining.saturating_add(1)), &mut out)
                    }
                    (OverrideFileSource::ArchiveEntry(_), None, _) => {
                        unreachable!("an archive entry is listed from an archive")
                    }
                    (OverrideFileSource::DirectoryFile, _, None) => {
                        unreachable!("a directory file is listed from a directory")
                    }
                }
                .map_err(|err| storage(&err))?;

                if written > remaining {
                    return Err(InstanceError::FileTooLarge {
                        path: file.relative_path,
                        size: copied + written,
                        limit: max_bytes,
                    });
                }
                copied += written;

                staged.push(StagedOverride {
                    relative_path: file.relative_path,
                    path,
                });
            }
        }

        Ok(staged)
    })
    .await
}

fn list_archive(
    archive_path: &Path,
    subdir: &str,
    max_files: usize,
) -> Result<Vec<OverrideFile>, InstanceError> {
    let mut archive = open_archive(archive_path)?;

    let subdir = subdir.replace('\\', "/");
    let subdir = subdir.trim_matches('/');
    let prefix = if subdir.is_empty() {
        String::new()
    } else {
        format!("{subdir}/")
    };

    let mut files = Vec::new();

    for index in 0..archive.len() {
        // Raw: only the header is read here, nothing is decompressed.
        let entry = archive
            .by_index_raw(index)
            .map_err(|err| invalid_archive(&err))?;
        let name = entry.name();

        // `enclosed_name` refuses a name that climbs out of the root, but takes an absolute
        // one (`/evil`, `C:\evil`) as relative; that is no name of a pack either.
        if entry.enclosed_name().is_none() || is_absolute(name) {
            return Err(InstanceError::InvalidRelativePath {
                path: name.to_owned(),
                reason: format!(
                    "the entry of \"{}\" escapes the archive",
                    archive_path.display()
                ),
            });
        }

        if entry.is_dir() || entry.is_symlink() {
            continue;
        }

        let Some(relative_path) = name.strip_prefix(&prefix) else {
            continue;
        };

        files.push(OverrideFile {
            relative_path: relative_path.to_owned(),
            declared_size: entry.size(),
            source: OverrideFileSource::ArchiveEntry(index),
        });
        if files.len() > max_files {
            break;
        }
    }

    Ok(files)
}

fn list_directory(root: &Path, max_files: usize) -> Result<Vec<OverrideFile>, InstanceError> {
    fn walk(
        dir: &Path,
        prefix: &str,
        max_files: usize,
        files: &mut Vec<OverrideFile>,
    ) -> Result<(), InstanceError> {
        let mut entries = std::fs::read_dir(dir)
            .map_err(|err| storage(&err))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| storage(&err))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);

        for entry in entries {
            if files.len() > max_files {
                break;
            }

            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| InstanceError::InvalidRelativePath {
                    path: entry.path().display().to_string(),
                    reason: "override file name is not valid UTF-8".to_owned(),
                })?;
            let relative_path = format!("{prefix}{name}");

            // `DirEntry::file_type` does not follow symlinks.
            let file_type = entry.file_type().map_err(|err| storage(&err))?;
            if file_type.is_dir() {
                walk(
                    &entry.path(),
                    &format!("{relative_path}/"),
                    max_files,
                    files,
                )?;
            } else if file_type.is_file() {
                let declared_size = entry.metadata().map_err(|err| storage(&err))?.len();
                files.push(OverrideFile {
                    relative_path,
                    declared_size,
                    source: OverrideFileSource::DirectoryFile,
                });
            } else {
                tracing::warn!("Skipping override \"{relative_path}\": not a regular file");
            }
        }

        Ok(())
    }

    let is_dir = std::fs::symlink_metadata(root).is_ok_and(|metadata| metadata.is_dir());
    if !is_dir {
        return Err(InstanceError::Storage(format!(
            "Overrides directory \"{}\" is not a directory",
            root.display()
        )));
    }

    let mut files = Vec::new();
    walk(root, "", max_files, &mut files)?;
    Ok(files)
}

/// A handle on the root of a directory source, its last component not followed either.
fn open_root(root: &Path) -> Result<Dir, InstanceError> {
    let opened = match (root.parent(), root.file_name()) {
        (Some(parent), Some(name)) if !parent.as_os_str().is_empty() => {
            Dir::open_ambient_dir(parent, ambient_authority())
                .and_then(|parent| parent.open_dir_nofollow(name))
        }
        _ => Dir::open_ambient_dir(root, ambient_authority()),
    };

    opened.map_err(|err| {
        InstanceError::Storage(format!(
            "Failed to open overrides directory \"{}\": {err}",
            root.display()
        ))
    })
}

/// Open the regular file at `relative_path` (`/`-separated) under `root`, every component
/// opened from the one before it without following a symlink: a symlink anywhere on the way
/// (a junction on Windows) fails the open instead of leading elsewhere.
fn open_in(root: &Dir, relative_path: &str) -> std::io::Result<File> {
    let invalid = |reason: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, reason);

    let mut components: Vec<&str> = relative_path.split('/').collect();
    if components
        .iter()
        .any(|component| matches!(*component, "" | "." | ".."))
    {
        return Err(invalid("not a plain relative path"));
    }
    let name = components
        .pop()
        .ok_or_else(|| invalid("not a plain relative path"))?;

    let mut dir: Option<Dir> = None;
    for component in components {
        let next = dir.as_ref().unwrap_or(root).open_dir_nofollow(component)?;
        dir = Some(next);
    }

    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = dir.as_ref().unwrap_or(root).open_with(name, &options)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("not a regular file"));
    }

    Ok(file.into_std())
}

/// Whether an archive entry's name is rooted or starts with a drive or other prefix, read with
/// either separator.
fn is_absolute(name: &str) -> bool {
    name.starts_with(['/', '\\'])
        || name
            .split(['/', '\\'])
            .next()
            .is_some_and(|first| first.contains(':'))
}

fn open_archive(archive_path: &Path) -> Result<zip::ZipArchive<File>, InstanceError> {
    let file = File::open(archive_path).map_err(|err| {
        InstanceError::Storage(format!(
            "Failed to open overrides archive \"{}\": {err}",
            archive_path.display()
        ))
    })?;

    zip::ZipArchive::new(file).map_err(|err| invalid_archive(&err))
}

fn invalid_archive(err: &zip::result::ZipError) -> InstanceError {
    InstanceError::ContentProviderError {
        reason: format!("Invalid overrides archive: {err}"),
    }
}

fn storage(err: &std::io::Error) -> InstanceError {
    InstanceError::Storage(err.to_string())
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, InstanceError> + Send + 'static,
) -> Result<T, InstanceError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|err| InstanceError::Storage(err.to_string()))?
}

#[cfg(test)]
#[path = "plan_overrides_test.rs"]
mod tests;
