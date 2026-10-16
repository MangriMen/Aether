//! Housekeeping for the `.mrpack` archives kept in the cache.
//!
//! An archive is only a cache: it is named by the hash it was verified against and can always be
//! downloaded again from the version's URL. What must not happen is its removal between the plan
//! that points at it and the executor reading its overrides. So an archive is judged by when it
//! was last *used*, not made: every use ([`touch`]) renews it, and only archives untouched for
//! [`MAX_AGE`] go — long after any plan that named them has ended.

use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use async_trait::async_trait;

use crate::shared::housekeeping::{CacheCleaner, CacheKind};

/// How long an archive may sit unused. An update of a pack reads the archive of the installed
/// version only as a convenience; after this long it is simply downloaded again.
pub(super) const MAX_AGE: Duration = Duration::from_hours(14 * 24);

/// Mark the archive at `path` as used just now.
pub(super) async fn touch(path: &Path) {
    let path = path.to_owned();
    let renewed = tokio::task::spawn_blocking(move || {
        std::fs::File::options()
            .write(true)
            .open(&path)?
            .set_modified(SystemTime::now())
    })
    .await;

    if !matches!(renewed, Ok(Ok(()))) {
        tracing::debug!("Failed to renew a cached modpack archive: {renewed:?}");
    }
}

/// Remove the archives in `dir` not used for [`MAX_AGE`] as of `now`, except `keep`. Best
/// effort: a file that cannot be removed stays for the next time.
///
/// Touches nothing but archives in `dir` itself (and the partial files an interrupted download
/// leaves beside them): a file of any other kind, or in a subdirectory, is not this cache's.
pub(super) async fn prune(dir: &Path, keep: &Path, now: SystemTime) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path == keep || !is_cache_file(&path) {
            continue;
        }

        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        let unused_for = metadata
            .modified()
            .ok()
            .and_then(|modified| now.duration_since(modified).ok());

        if metadata.is_file()
            && unused_for.is_some_and(|age| age >= MAX_AGE)
            && let Err(err) = tokio::fs::remove_file(&path).await
        {
            tracing::debug!("Failed to remove {}: {err}", path.display());
        }
    }
}

/// An archive, or the partial file of a download that never finished.
fn is_cache_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mrpack"))
        || (name.starts_with(".download-")
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("part")))
}

/// Empties the folder of archives by the user's wish. An archive is downloaded again when a
/// plan needs it; the partial file of a download that may be running is left to it.
pub struct MrpackCacheCleaner {
    dir: PathBuf,
}

impl MrpackCacheCleaner {
    /// The archives in `dir`: [`ModrinthContentProvider`](super::ModrinthContentProvider) keeps
    /// them in [`mrpack_dir`].
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

#[async_trait]
impl CacheCleaner for MrpackCacheCleaner {
    fn what(&self) -> &'static str {
        "modpack archives"
    }

    fn kind(&self) -> CacheKind {
        CacheKind::ModpackArchives
    }

    async fn size(&self) -> u64 {
        crate::shared::housekeeping::files_size(&self.dir).await
    }

    async fn clear(&self) -> u64 {
        let Ok(mut entries) = tokio::fs::read_dir(&self.dir).await else {
            return 0;
        };

        let mut removed = 0;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let is_archive = path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("mrpack"));
            if !is_archive || !entry.metadata().await.is_ok_and(|m| m.is_file()) {
                continue;
            }
            match tokio::fs::remove_file(&path).await {
                Ok(()) => removed += 1,
                Err(err) => tracing::debug!("Failed to remove {}: {err}", path.display()),
            }
        }
        removed
    }
}

/// Where the archives are kept, under the cache folder of the app.
pub fn mrpack_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join("modrinth").join("mrpack")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, age: Duration) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, name).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(SystemTime::now() - age)
            .unwrap();
        path
    }

    const OLD: Duration = Duration::from_hours(30 * 24);
    const FRESH: Duration = Duration::from_mins(1);

    #[tokio::test]
    async fn should_remove_only_archives_when_they_were_unused_for_long() {
        let dir = tempfile::tempdir().unwrap();
        let old = write(dir.path(), "old.mrpack", OLD);
        let fresh = write(dir.path(), "fresh.mrpack", FRESH);
        let partial = write(dir.path(), ".download-abc.part", OLD);

        prune(dir.path(), Path::new("none"), SystemTime::now()).await;

        assert!(!old.exists());
        assert!(!partial.exists());
        assert!(fresh.exists());
    }

    #[tokio::test]
    async fn should_keep_the_archive_a_plan_points_at_even_when_old() {
        let dir = tempfile::tempdir().unwrap();
        let current = write(dir.path(), "current.mrpack", OLD);

        prune(dir.path(), &current, SystemTime::now()).await;

        assert!(current.exists());
    }

    #[tokio::test]
    async fn should_keep_an_old_archive_when_it_was_used_again() {
        let dir = tempfile::tempdir().unwrap();
        let reused = write(dir.path(), "reused.mrpack", OLD);

        touch(&reused).await;
        prune(dir.path(), Path::new("none"), SystemTime::now()).await;

        assert!(reused.exists());
    }

    #[tokio::test]
    async fn should_leave_a_file_alone_when_it_is_not_an_archive() {
        let dir = tempfile::tempdir().unwrap();
        let notes = write(dir.path(), "notes.txt", OLD);
        let jar = write(dir.path(), "mod.jar", OLD);
        std::fs::create_dir(dir.path().join("folder.mrpack")).unwrap();
        let nested = write(&dir.path().join("folder.mrpack"), "inner.mrpack", OLD);

        prune(dir.path(), Path::new("none"), SystemTime::now()).await;

        assert!(notes.exists());
        assert!(jar.exists());
        assert!(dir.path().join("folder.mrpack").is_dir());
        assert!(nested.exists());
    }

    #[tokio::test]
    async fn should_not_fail_when_the_directory_is_missing() {
        prune(
            Path::new("definitely/not/here"),
            Path::new("none"),
            SystemTime::now(),
        )
        .await;
    }

    #[tokio::test]
    async fn should_remove_every_archive_but_not_other_files_when_cleared() {
        let dir = tempfile::tempdir().unwrap();
        let fresh = write(dir.path(), "fresh.mrpack", FRESH);
        let old = write(dir.path(), "old.mrpack", OLD);
        let partial = write(dir.path(), ".download-abc.part", FRESH);
        let notes = write(dir.path(), "notes.txt", OLD);

        let removed = MrpackCacheCleaner::new(dir.path().to_owned()).clear().await;

        assert_eq!(removed, 2);
        assert!(!fresh.exists());
        assert!(!old.exists());
        assert!(partial.exists());
        assert!(notes.exists());
    }
}
