//! Reading the overrides of an install plan and copying them out into staging; writing them into
//! the instance is the executor's, through the same checks as the planned files.
//!
//! Paths are the source's own, `/`-separated, never parsed as host paths, so an entry means the
//! same file on every platform.
//!
//! Security: an archive with any entry escaping its root (`../evil`, an absolute path) is refused
//! as a whole. Symlinks are never materialised, as they could point out of the instance or the
//! source. Every source is an [`AnchoredPath`]: its root, which the core trusts (the mount root
//! for a path a plugin names), is opened by its path, and from there every component of the
//! source, and of each file of a directory source, is opened from a handle on the one before it
//! without following a symlink (a junction on Windows). A component swapped for a link at any
//! time, after the plugin's path was checked or after the source was listed, fails the open
//! instead of leading out of the root ([`open_file_nofollow_in`]). On Windows cap-std's
//! `Dir::entries` lists by the path of the open handle, so what a listing returns is not trusted
//! for safety; only the later no-follow open from the anchor is. What an archive unpacks to is
//! bounded over all sources of a plan together (`ContentInstallLimits`): files when listed, bytes
//! as staged, so a lying header does not help.

use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, copy},
    path::{Component, Path, PathBuf},
};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::{ambient_authority, fs::Dir};

use crate::features::instance::{AnchoredPath, InstanceError, OverrideLocation};

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
                OverrideLocation::Directory { path } => (None, Some(open_source_dir(path)?)),
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
    archive_path: &AnchoredPath,
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
        let name = entry.name().map_err(|err| invalid_archive(&err))?;
        let name = name.as_ref();

        // `enclosed_name` refuses a name that climbs out of the root, but takes an absolute
        // one (`/evil`, `C:\evil`) as relative; that is no name of a pack either.
        if entry.enclosed_name().is_none() || is_absolute(name) {
            return Err(InstanceError::InvalidRelativePath {
                path: name.to_owned(),
                reason: format!("the entry of \"{archive_path}\" escapes the archive"),
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

fn list_directory(
    root: &AnchoredPath,
    max_files: usize,
) -> Result<Vec<OverrideFile>, InstanceError> {
    fn walk(
        dir: &Dir,
        prefix: &str,
        max_files: usize,
        files: &mut Vec<OverrideFile>,
    ) -> Result<(), InstanceError> {
        let mut entries = dir
            .entries()
            .map_err(|err| storage(&err))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| storage(&err))?;
        entries.sort_by_key(cap_std::fs::DirEntry::file_name);

        for entry in entries {
            if files.len() > max_files {
                break;
            }

            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| InstanceError::InvalidRelativePath {
                    path: format!("{prefix}{}", name.to_string_lossy()),
                    reason: "override file name is not valid UTF-8".to_owned(),
                })?;
            let relative_path = format!("{prefix}{name}");

            // `DirEntry::file_type` does not follow symlinks.
            let file_type = entry.file_type().map_err(|err| storage(&err))?;
            if file_type.is_dir() {
                let subdir = dir.open_dir_nofollow(name).map_err(|err| storage(&err))?;
                walk(&subdir, &format!("{relative_path}/"), max_files, files)?;
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

    let dir = open_source_dir(root)?;
    let mut files = Vec::new();
    walk(&dir, "", max_files, &mut files)?;
    Ok(files)
}

/// A handle on a directory source, reached from its root without following a symlink.
fn open_source_dir(path: &AnchoredPath) -> Result<Dir, InstanceError> {
    let opened = open_root(path.root()).and_then(|root| {
        let names = plain_components(path.relative())?;
        Ok(open_dirs_nofollow(&root, &names)?.unwrap_or(root))
    });

    opened.map_err(|err| {
        InstanceError::Storage(format!(
            "Failed to open overrides directory \"{path}\": {err}"
        ))
    })
}

/// Open an archive source, reached from its root without following a symlink.
fn open_archive(archive_path: &AnchoredPath) -> Result<zip::ZipArchive<File>, InstanceError> {
    let file = open_root(archive_path.root())
        .and_then(|root| open_file_nofollow_in(&root, archive_path.relative()))
        .map_err(|err| {
            InstanceError::Storage(format!(
                "Failed to open overrides archive \"{archive_path}\": {err}"
            ))
        })?;

    zip::ZipArchive::new(file).map_err(|err| invalid_archive(&err))
}

/// The root of an [`AnchoredPath`], opened by its path: the core trusts it, links and all.
fn open_root(root: &Path) -> std::io::Result<Dir> {
    Dir::open_ambient_dir(root, ambient_authority())
}

/// Open the regular file at `relative_path` (`/`-separated, as listed) under `root`.
fn open_in(root: &Dir, relative_path: &str) -> std::io::Result<File> {
    if relative_path
        .split('/')
        .any(|component| matches!(component, "" | "." | ".."))
    {
        return Err(invalid_input("not a plain relative path"));
    }

    open_file_nofollow_in(root, Path::new(relative_path))
}

/// Open the regular file at `path` under `dir`, every component opened from a handle on the one
/// before it without following a symlink: a symlink anywhere on the way (a junction on Windows)
/// fails the open instead of leading elsewhere.
fn open_file_nofollow_in(dir: &Dir, path: &Path) -> std::io::Result<File> {
    let names = plain_components(path)?;
    let (name, parents) = names
        .split_last()
        .ok_or_else(|| invalid_input("not a file path"))?;

    let parent = open_dirs_nofollow(dir, parents)?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = parent.as_ref().unwrap_or(dir).open_with(name, &options)?;
    if !file.metadata()?.is_file() {
        return Err(invalid_input("not a regular file"));
    }

    Ok(file.into_std())
}

/// The directory at `names` under `dir`, each opened from the one before without following a
/// symlink; `None` for no names, `dir` itself.
fn open_dirs_nofollow(dir: &Dir, names: &[&OsStr]) -> std::io::Result<Option<Dir>> {
    let mut opened: Option<Dir> = None;
    for name in names {
        let next = opened.as_ref().unwrap_or(dir).open_dir_nofollow(name)?;
        opened = Some(next);
    }
    Ok(opened)
}

/// The names `path` is made of, refused unless each is a plain name.
fn plain_components(path: &Path) -> std::io::Result<Vec<&OsStr>> {
    path.components()
        .map(|component| match component {
            Component::Normal(name) => Ok(name),
            _ => Err(invalid_input("not a plain relative path")),
        })
        .collect()
}

fn invalid_input(reason: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, reason.to_owned())
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
