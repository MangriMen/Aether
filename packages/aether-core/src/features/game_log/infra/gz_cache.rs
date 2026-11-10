use std::path::{Path, PathBuf};

use tokio::fs;

use crate::{
    features::game_log::domain::{GameLogError, LogSourceRef},
    shared::io::domain::IoError,
};

use super::fs_store::modified_millis;

/// A packed log that unpacks to more than this is refused, so a crafted file cannot fill the disk.
const MAX_UNPACKED_BYTES: u64 = 256 * 1024 * 1024;

/// The unpacked copy of a `.gz` log in `cache_dir`, unpacking it once per version of the file.
pub(super) async fn unpacked_path(
    cache_dir: PathBuf,
    source: &LogSourceRef,
    path: PathBuf,
    meta: &std::fs::Metadata,
) -> Result<PathBuf, GameLogError> {
    let name = format!(
        "{}-{}-{}.log",
        source.name(),
        meta.len(),
        modified_millis(meta).unwrap_or(0)
    );
    let target = cache_dir.join(name);
    if fs::try_exists(&target).await.unwrap_or(false) {
        return Ok(target);
    }

    fs::create_dir_all(&cache_dir)
        .await
        .map_err(|err| IoError::with_path(err, &cache_dir))?;
    remove_stale_unpacked(&cache_dir, source.name(), &target).await;
    let source_id = source.id();
    let target_for_task = target.clone();
    tokio::task::spawn_blocking(move || unpack(&path, &target_for_task, &source_id))
        .await
        .map_err(|err| IoError::from(std::io::Error::other(err)))??;
    Ok(target)
}

/// Drops the unpacked copies of earlier versions of the same log, so the cache keeps one per log.
async fn remove_stale_unpacked(cache_dir: &Path, name: &str, keep: &Path) {
    let prefix = format!("{name}-");
    let Ok(mut entries) = fs::read_dir(cache_dir).await else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        // A `.part` file is another read's unpack in progress, which that read renames itself.
        let is_old_copy = file_name.starts_with(&prefix)
            && Path::new(&file_name)
                .extension()
                .is_none_or(|ext| ext != "part")
            && entry.path() != keep;
        if is_old_copy {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
}

fn unpack(path: &Path, target: &Path, source_id: &str) -> Result<(), GameLogError> {
    use std::io::Read;

    // Unique, so two reads of the same log unpack side by side instead of over each other.
    let part = target.with_extension(format!("{}.part", uuid::Uuid::new_v4()));
    let result = (|| -> std::io::Result<u64> {
        let input = std::fs::File::open(path)?;
        let mut decoder = flate2::read::GzDecoder::new(input).take(MAX_UNPACKED_BYTES + 1);
        let mut output = std::fs::File::create(&part)?;
        std::io::copy(&mut decoder, &mut output)
    })();

    let finish = |part: &Path| {
        let _ = std::fs::remove_file(part);
    };
    match result {
        Ok(written) if written > MAX_UNPACKED_BYTES => {
            finish(&part);
            Err(GameLogError::TooLarge {
                source_id: source_id.to_string(),
                limit: MAX_UNPACKED_BYTES,
            })
        }
        Ok(_) => std::fs::rename(&part, target).map_err(|err| {
            finish(&part);
            IoError::with_path(err, target).into()
        }),
        Err(err) => {
            finish(&part);
            Err(IoError::with_path(err, path).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_keep_one_unpacked_copy_when_the_log_changes() {
        let cache = tempfile::tempdir().unwrap();
        let old = cache.path().join("a.log.gz-10-1.log");
        let other = cache.path().join("b.log.gz-10-1.log");
        let current = cache.path().join("a.log.gz-20-2.log");
        let unpacking = cache.path().join("a.log.gz-20-2.1234.part");
        for path in [&old, &other, &current, &unpacking] {
            std::fs::write(path, "x").unwrap();
        }

        remove_stale_unpacked(cache.path(), "a.log.gz", &current).await;

        assert!(!old.exists());
        assert!(other.exists());
        assert!(current.exists());
        assert!(unpacking.exists());
    }
}
