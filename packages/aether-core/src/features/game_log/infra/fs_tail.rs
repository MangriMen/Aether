use std::{io::SeekFrom, path::PathBuf, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
    sync::oneshot,
    time::MissedTickBehavior,
};

use crate::features::{
    events::{EventEmitterExt, GameLogEvent, SharedEventEmitter},
    game_log::{
        app::{GameLogTailService, LogTail},
        domain::{LogSourceKind, LogSourceRef},
    },
    instance::InstanceId,
    settings::LocationInfo,
};

const LATEST_LOG: &str = "latest.log";
/// The file is looked at this often; the game writes it on its own, with no event to wait for.
const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Looked at to find where the last whole line of the file starts.
const START_LOOKBACK_BYTES: u64 = 64 * 1024;
/// The most one announcement carries; more that was written is announced by the next one.
const MAX_EVENT_TEXT_BYTES: u64 = 256 * 1024;

pub struct FsGameLogTail {
    location_info: Arc<LocationInfo>,
    event_emitter: SharedEventEmitter,
}

impl FsGameLogTail {
    pub fn new(location_info: Arc<LocationInfo>, event_emitter: SharedEventEmitter) -> Self {
        Self {
            location_info,
            event_emitter,
        }
    }
}

#[async_trait]
impl GameLogTailService for FsGameLogTail {
    async fn follow(&self, instance_id: &InstanceId) -> LogTail {
        let (tail, stop, done) = LogTail::new();
        let Ok(source) = LogSourceRef::new(LogSourceKind::Game, LATEST_LOG) else {
            return tail;
        };
        let path = self.location_info.logs_dir(instance_id).join(LATEST_LOG);
        // Found before the following is handed back, so that a line written right after is not missed.
        let at = first_position(&path).await;
        let follower = Follower {
            path,
            instance_id: instance_id.clone(),
            source_id: source.id(),
            emitter: self.event_emitter.clone(),
        };

        tokio::spawn(async move {
            follower.run(at, stop).await;
            let _ = done.send(());
        });
        tail
    }
}

struct Follower {
    path: PathBuf,
    instance_id: InstanceId,
    source_id: String,
    emitter: SharedEventEmitter,
}

impl Follower {
    async fn run(self, mut at: u64, mut stop: oneshot::Receiver<()>) {
        let mut tick = tokio::time::interval(POLL_INTERVAL);
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            let stopping = tokio::select! {
                _ = &mut stop => true,
                _ = tick.tick() => false,
            };
            // A last look at the file after the stop, for what the game wrote as it ended.
            at = self.announce_new_lines(at).await;
            if stopping {
                return;
            }
        }
    }

    /// Announces the whole lines written since `at`; returns where the next ones begin. A file
    /// shorter than `at` is a new one, the game having rotated the log: it is read from its start.
    async fn announce_new_lines(&self, at: u64) -> u64 {
        // Not there yet, or not readable now: looked at again.
        let Ok((bytes, size)) = read_from(&self.path, at, MAX_EVENT_TEXT_BYTES).await else {
            return at;
        };
        if size < at {
            tracing::info!(
                instance_id = %self.instance_id,
                "The game replaced its latest.log: following the new one"
            );
            return 0;
        }

        let cut = bytes.len() as u64 == MAX_EVENT_TEXT_BYTES;
        let whole = announced_len(&bytes, cut);
        if whole == 0 {
            return at;
        }
        let end = at + whole as u64;
        self.emitter
            .emit_safe(GameLogEvent {
                instance_id: self.instance_id.clone(),
                source_id: self.source_id.clone(),
                start: at,
                end,
                text: Arc::from(String::from_utf8_lossy(&bytes[..whole])),
            })
            .await;
        end
    }
}

/// Up to `max` bytes of the file from `from`, and the size of the file. Bytes, not text: a
/// position in the file is a count of bytes.
async fn read_from(path: &PathBuf, from: u64, max: u64) -> std::io::Result<(Vec<u8>, u64)> {
    let mut file = File::open(path).await?;
    let size = file.metadata().await?.len();
    let from = from.min(size);
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(from)).await?;
    file.take(max).read_to_end(&mut bytes).await?;
    Ok((bytes, size))
}

/// Where the lines to announce begin: the line the file ends in, so that a line the game has
/// half written is announced whole.
async fn first_position(path: &PathBuf) -> u64 {
    let Ok((_, size)) = read_from(path, 0, 0).await else {
        return 0;
    };
    let from = size.saturating_sub(START_LOOKBACK_BYTES);
    let Ok((bytes, size)) = read_from(path, from, START_LOOKBACK_BYTES).await else {
        return 0;
    };
    match bytes.iter().rposition(|&b| b == b'\n') {
        Some(pos) => from + pos as u64 + 1,
        None if from > 0 => size,
        None => 0,
    }
}

/// How many leading bytes are announced: up to the last line break, a line still being written
/// being left out. When `cut` (the read stopped at its limit) and there is no line break at
/// all, the line is longer than the limit and is announced in pieces, each cut at a character.
fn announced_len(bytes: &[u8], cut: bool) -> usize {
    if let Some(pos) = bytes.iter().rposition(|&b| b == b'\n') {
        return pos + 1;
    }
    if !cut {
        return 0;
    }
    match std::str::from_utf8(bytes) {
        Err(err) if err.error_len().is_none() => err.valid_up_to(),
        Ok(_) | Err(_) => bytes.len(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tokio::sync::mpsc;

    use super::*;
    use crate::features::events::{Event, test_support::ChannelEmitter};

    struct Rig {
        _location: tempfile::TempDir,
        log: PathBuf,
        tail: FsGameLogTail,
        id: InstanceId,
        events: mpsc::UnboundedReceiver<Event>,
    }

    fn rig(initial: &str) -> Rig {
        let location = tempfile::tempdir().unwrap();
        let info = Arc::new(LocationInfo::new(
            location.path().join("config"),
            location.path().join("cache"),
        ));
        let id = InstanceId::try_from("inst".to_string()).unwrap();
        std::fs::create_dir_all(info.logs_dir(&id)).unwrap();
        let log = info.logs_dir(&id).join(LATEST_LOG);
        std::fs::write(&log, initial).unwrap();
        let (tx, events) = mpsc::unbounded_channel();
        let emitter: SharedEventEmitter = Arc::new(ChannelEmitter(tx));
        Rig {
            _location: location,
            log,
            tail: FsGameLogTail::new(info, emitter),
            id,
            events,
        }
    }

    fn append(path: &std::path::Path, text: &str) {
        let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    async fn next_event(events: &mut mpsc::UnboundedReceiver<Event>) -> GameLogEvent {
        let event = tokio::time::timeout(Duration::from_secs(5), events.recv())
            .await
            .expect("an event in time")
            .unwrap();
        let Event::GameLog(event) = event else {
            panic!("expected a game log event");
        };
        event
    }

    #[tokio::test]
    async fn should_announce_only_the_lines_written_after_the_following_began() {
        let mut rig = rig("old line\n");
        let tail = rig.tail.follow(&rig.id).await;

        append(&rig.log, "new line\n");
        let event = next_event(&mut rig.events).await;

        assert_eq!(&*event.text, "new line\n");
        assert_eq!((event.start, event.end), (9, 18));
        assert_eq!(event.source_id, "game:latest.log");
        tail.finish(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn should_announce_a_line_whole_when_it_is_written_in_parts() {
        let mut rig = rig("");
        let tail = rig.tail.follow(&rig.id).await;

        append(&rig.log, "half");
        tokio::time::sleep(POLL_INTERVAL * 2).await;
        append(&rig.log, " done\n");
        let event = next_event(&mut rig.events).await;

        assert_eq!(&*event.text, "half done\n");
        tail.finish(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn should_look_once_more_after_it_is_told_to_stop() {
        let mut rig = rig("");
        let tail = rig.tail.follow(&rig.id).await;

        append(&rig.log, "last words\n");
        tail.finish(Duration::from_secs(5)).await;

        let event = next_event(&mut rig.events).await;
        assert_eq!(&*event.text, "last words\n");
    }

    #[tokio::test]
    async fn should_follow_a_log_that_appears_after_the_following_began() {
        let mut rig = rig("x\n");
        std::fs::remove_file(&rig.log).unwrap();
        let tail = rig.tail.follow(&rig.id).await;

        std::fs::write(&rig.log, "fresh\n").unwrap();
        let event = next_event(&mut rig.events).await;

        assert_eq!(&*event.text, "fresh\n");
        tail.finish(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn should_follow_the_new_log_from_its_start_when_the_game_replaces_it_with_a_shorter_one()
    {
        let mut rig = rig("a long first line\n");
        let tail = rig.tail.follow(&rig.id).await;

        std::fs::write(&rig.log, "short\n").unwrap();
        let event = next_event(&mut rig.events).await;

        assert_eq!(&*event.text, "short\n");
        assert_eq!((event.start, event.end), (0, 6));
        tail.finish(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn should_count_bytes_not_characters_when_the_log_holds_bytes_that_are_not_utf8() {
        let mut rig = rig("");
        let tail = rig.tail.follow(&rig.id).await;

        // cp1251 for a word, a line break, and the start of a line still being written.
        std::fs::write(&rig.log, b"\xcf\xf0\xe8\xe2\xe5\xf2\npart").unwrap();
        let event = next_event(&mut rig.events).await;
        assert_eq!((event.start, event.end), (0, 7));

        append(&rig.log, "ial\n");
        let event = next_event(&mut rig.events).await;
        assert_eq!(&*event.text, "partial\n");
        assert_eq!((event.start, event.end), (7, 15));
        tail.finish(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn should_announce_a_line_longer_than_one_announcement_in_pieces() {
        let mut rig = rig("");
        let tail = rig.tail.follow(&rig.id).await;
        let long = "é".repeat(usize::try_from(MAX_EVENT_TEXT_BYTES).unwrap() / 2 + 10);

        append(&rig.log, &long);
        let first = next_event(&mut rig.events).await;
        append(&rig.log, "\n");
        let second = next_event(&mut rig.events).await;

        assert_eq!(first.start, 0);
        assert_eq!(first.end, MAX_EVENT_TEXT_BYTES);
        assert_eq!(second.start, first.end);
        assert_eq!(
            format!("{}{}", first.text, second.text),
            format!("{long}\n")
        );
        tail.finish(Duration::from_secs(5)).await;
    }
}
