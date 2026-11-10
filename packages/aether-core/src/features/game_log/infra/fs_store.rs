use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use async_trait::async_trait;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncSeekExt},
};

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

/// A packed log that unpacks to more than this is refused, so a crafted file cannot fill the disk.
const MAX_UNPACKED_BYTES: u64 = 256 * 1024 * 1024;
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
        let source_id = source.id();
        let target_for_task = target.clone();
        tokio::task::spawn_blocking(move || unpack(&path, &target_for_task, &source_id))
            .await
            .map_err(|err| IoError::from(std::io::Error::other(err)))??;
        Ok(target)
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

fn modified_millis(meta: &std::fs::Metadata) -> Option<i64> {
    let since_epoch = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(since_epoch.as_millis()).ok()
}

fn unpack(path: &Path, target: &Path, source_id: &str) -> Result<(), GameLogError> {
    use std::io::Read;

    let part = target.with_extension("part");
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

async fn read_page(
    path: &Path,
    cursor: LogCursor,
    max_bytes: u64,
) -> Result<LogPage, GameLogError> {
    let io = |err| IoError::with_path(err, path);
    let mut file = File::open(path).await.map_err(io)?;
    let size = file.metadata().await.map_err(io)?.len();

    let (start, end) = match cursor {
        LogCursor::Tail => (size.saturating_sub(max_bytes), size),
        LogCursor::Before(at) => {
            let end = at.min(size);
            (end.saturating_sub(max_bytes), end)
        }
        LogCursor::After(at) => {
            let start = at.min(size);
            (start, start.saturating_add(max_bytes).min(size))
        }
    };

    let mut buf = vec![0u8; usize::try_from(end - start).unwrap_or(0)];
    file.seek(SeekFrom::Start(start)).await.map_err(io)?;
    file.read_exact(&mut buf).await.map_err(io)?;

    // Positions sit on line starts, so only an edge cut by the page size can split a line: drop
    // the cut part (it belongs to the next page) unless that would leave nothing.
    let (mut lo, mut hi) = (0, buf.len());
    let after = matches!(cursor, LogCursor::After(_));
    if after && end < size {
        hi = buf
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(hi, |pos| pos + 1);
    }
    if !after && start > 0 {
        lo = buf
            .iter()
            .position(|&b| b == b'\n')
            .filter(|pos| pos + 1 < buf.len())
            .map_or(0, |pos| pos + 1);
    }

    Ok(LogPage {
        text: String::from_utf8_lossy(&buf[lo..hi]).into_owned(),
        start: start + lo as u64,
        end: start + hi as u64,
        has_more_before: start + lo as u64 > 0,
        has_more_after: start + (hi as u64) < size,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn file_with(content: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("latest.log");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn numbered(lines: usize) -> Vec<u8> {
        (0..lines)
            .flat_map(|i| format!("line {i:04}\n").into_bytes())
            .collect()
    }

    #[tokio::test]
    async fn should_return_whole_lines_ending_at_the_end_when_reading_the_tail() {
        let (_dir, path) = file_with(&numbered(1000));

        let page = read_page(&path, LogCursor::Tail, 1000).await.unwrap();

        assert!(page.text.starts_with("line "));
        assert!(page.text.ends_with("line 0999\n"));
        assert_eq!(page.end, 10_000);
        assert!(page.has_more_before);
        assert!(!page.has_more_after);
        assert_eq!(page.text.len() as u64, page.end - page.start);
    }

    #[tokio::test]
    async fn should_continue_without_gaps_when_paging_forward_to_the_end() {
        let content = numbered(1000);
        let (_dir, path) = file_with(&content);

        let mut text = String::new();
        let mut at = 0;
        loop {
            let page = read_page(&path, LogCursor::After(at), 1000).await.unwrap();
            assert_eq!(page.start, at);
            text.push_str(&page.text);
            at = page.end;
            if !page.has_more_after {
                break;
            }
        }

        assert_eq!(text.as_bytes(), content.as_slice());
    }

    #[tokio::test]
    async fn should_continue_without_gaps_when_paging_backward_to_the_start() {
        let content = numbered(1000);
        let (_dir, path) = file_with(&content);

        let mut pages = Vec::new();
        let mut page = read_page(&path, LogCursor::Tail, 1000).await.unwrap();
        loop {
            pages.push(page.text.clone());
            if !page.has_more_before {
                break;
            }
            page = read_page(&path, LogCursor::Before(page.start), 1000)
                .await
                .unwrap();
        }

        pages.reverse();
        assert_eq!(pages.concat().as_bytes(), content.as_slice());
    }

    #[tokio::test]
    async fn should_read_nothing_new_when_the_cursor_is_at_the_end() {
        let (_dir, path) = file_with(&numbered(3));

        let page = read_page(&path, LogCursor::After(30), 1000).await.unwrap();

        assert_eq!(page.text, "");
        assert!(!page.has_more_after);
    }

    #[tokio::test]
    async fn should_keep_a_line_longer_than_the_page_when_it_has_no_newline() {
        let (_dir, path) = file_with(&vec![b'a'; 5000]);

        let page = read_page(&path, LogCursor::Tail, 1000).await.unwrap();

        assert_eq!(page.text.len(), 1000);
        assert!(page.has_more_before);
    }

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
