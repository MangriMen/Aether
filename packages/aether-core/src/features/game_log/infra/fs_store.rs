use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use async_trait::async_trait;
use tokio::fs;

use crate::{
    features::{
        game_log::{
            app::GameLogStore,
            domain::{GameLogError, LogCursor, LogPage, LogSource, LogSourceKind, LogSourceRef},
        },
        instance::InstanceId,
        settings::LocationInfo,
    },
    shared::io::domain::IoError,
};

use super::{gz_cache, page::read_page};

const LATEST_LOG: &str = "latest.log";

pub struct FsGameLogStore {
    location_info: Arc<LocationInfo>,
}

impl FsGameLogStore {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn dir_of(&self, instance_id: &InstanceId, kind: LogSourceKind) -> PathBuf {
        match kind {
            LogSourceKind::Console => self.location_info.game_logs_dir(instance_id),
            LogSourceKind::Game => self.location_info.logs_dir(instance_id),
        }
    }

    /// A regular file of the log folder; a link is refused so that the read cannot leave it.
    async fn resolve(
        &self,
        instance_id: &InstanceId,
        source: &LogSourceRef,
    ) -> Result<(PathBuf, std::fs::Metadata), GameLogError> {
        let path = self.dir_of(instance_id, source.kind()).join(source.name());
        let not_found = || GameLogError::SourceNotFound {
            source_id: source.id(),
        };

        let meta = fs::symlink_metadata(&path).await.map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                not_found()
            } else {
                IoError::with_path(err, &path).into()
            }
        })?;
        if !meta.is_file() {
            return Err(not_found());
        }
        Ok((path, meta))
    }

    /// A readable plain-text path for the source, unpacking a `.gz` once per version of the file.
    async fn readable_path(
        &self,
        instance_id: &InstanceId,
        source: &LogSourceRef,
        path: PathBuf,
        meta: &std::fs::Metadata,
    ) -> Result<PathBuf, GameLogError> {
        if !source.is_compressed() {
            return Ok(path);
        }

        let cache_dir = self
            .location_info
            .temp_dir()
            .join("game-logs")
            .join(instance_id.as_str());
        gz_cache::unpacked_path(cache_dir, source, path, meta).await
    }
}

#[async_trait]
impl GameLogStore for FsGameLogStore {
    async fn list(&self, instance_id: &InstanceId) -> Result<Vec<LogSource>, GameLogError> {
        let mut consoles = list_dir(
            &self.dir_of(instance_id, LogSourceKind::Console),
            LogSourceKind::Console,
        )
        .await?;
        consoles.sort_by(|a, b| b.source.name().cmp(a.source.name()));

        let mut game = list_dir(
            &self.dir_of(instance_id, LogSourceKind::Game),
            LogSourceKind::Game,
        )
        .await?;
        game.sort_by(|a, b| {
            let latest = |s: &LogSource| s.source.name() == LATEST_LOG;
            latest(b)
                .cmp(&latest(a))
                .then_with(|| b.modified.cmp(&a.modified))
                .then_with(|| b.source.name().cmp(a.source.name()))
        });

        consoles.extend(game);
        Ok(consoles)
    }

    async fn read(
        &self,
        instance_id: &InstanceId,
        source: &LogSourceRef,
        cursor: LogCursor,
        max_bytes: u64,
    ) -> Result<LogPage, GameLogError> {
        let (path, meta) = self.resolve(instance_id, source).await?;
        let path = self.readable_path(instance_id, source, path, &meta).await?;
        read_page(&path, cursor, max_bytes).await
    }
}

async fn list_dir(dir: &Path, kind: LogSourceKind) -> Result<Vec<LogSource>, GameLogError> {
    let mut entries = match fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(IoError::with_path(err, dir).into()),
    };

    let mut sources = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| IoError::with_path(err, dir))?
    {
        let is_file = entry.file_type().await.is_ok_and(|t| t.is_file());
        let name = entry.file_name().to_string_lossy().into_owned();
        // A name that is not a log (or not valid) is simply not offered.
        let Ok(source) = LogSourceRef::new(kind, name) else {
            continue;
        };
        if !is_file {
            continue;
        }
        let Ok(meta) = entry.metadata().await else {
            continue;
        };
        sources.push(LogSource {
            source,
            size: meta.len(),
            modified: modified_millis(&meta),
        });
    }
    Ok(sources)
}

pub(super) fn modified_millis(meta: &std::fs::Metadata) -> Option<i64> {
    let since_epoch = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(since_epoch.as_millis()).ok()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[tokio::test]
    async fn should_unpack_a_gz_once_when_it_is_listed_and_read() {
        let location = tempfile::tempdir().unwrap();
        let li = Arc::new(LocationInfo::new(
            location.path().join("config"),
            location.path().join("cache"),
        ));
        let id = InstanceId::try_from("inst".to_string()).unwrap();
        let logs = li.logs_dir(&id);
        std::fs::create_dir_all(&logs).unwrap();
        let gz = std::fs::File::create(logs.join("2026-01-01-1.log.gz")).unwrap();
        let mut enc = flate2::write::GzEncoder::new(gz, flate2::Compression::default());
        enc.write_all(b"packed line\n").unwrap();
        enc.finish().unwrap();
        let store = FsGameLogStore::new(li);

        let sources = store.list(&id).await.unwrap();
        let page = store
            .read(&id, &sources[0].source, LogCursor::Tail, 4096)
            .await
            .unwrap();

        assert_eq!(sources.len(), 1);
        assert_eq!(page.text, "packed line\n");
    }

    #[tokio::test]
    async fn should_list_console_logs_first_and_skip_files_that_are_not_logs() {
        let location = tempfile::tempdir().unwrap();
        let li = Arc::new(LocationInfo::new(
            location.path().join("config"),
            location.path().join("cache"),
        ));
        let id = InstanceId::try_from("inst".to_string()).unwrap();
        std::fs::create_dir_all(li.logs_dir(&id)).unwrap();
        std::fs::create_dir_all(li.game_logs_dir(&id)).unwrap();
        std::fs::write(li.logs_dir(&id).join("latest.log"), "x").unwrap();
        std::fs::write(li.logs_dir(&id).join("notes.txt"), "x").unwrap();
        std::fs::write(
            li.game_logs_dir(&id)
                .join("console-20260101-000000-000.log"),
            "x",
        )
        .unwrap();
        let store = FsGameLogStore::new(li);

        let ids: Vec<String> = store
            .list(&id)
            .await
            .unwrap()
            .iter()
            .map(|s| s.source.id())
            .collect();

        assert_eq!(
            ids,
            ["console:console-20260101-000000-000.log", "game:latest.log"]
        );
    }
}
