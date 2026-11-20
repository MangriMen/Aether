use std::{
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use tokio::{sync::oneshot, time::MissedTickBehavior};

use super::page::read_page;
use crate::features::{
    events::{EventEmitterExt, GameLogEvent, SharedEventEmitter},
    game_log::{
        app::{GameLogTailService, LogTail},
        domain::{LogCursor, LogSourceKind, LogSourceRef},
    },
    instance::InstanceId,
    settings::LocationInfo,
};

const LATEST_LOG: &str = "latest.log";
/// The file is looked at this often; the game writes it on its own, with no event to wait for.
const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Looked at to find where the last whole line of the file starts.
const START_LOOKBACK_BYTES: u64 = 64 * 1024;
/// A batch above this is announced by its range only; the UI reads it.
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

impl GameLogTailService for FsGameLogTail {
    fn follow(&self, instance_id: &InstanceId) -> LogTail {
        let (tail, stop, done) = LogTail::new();
        let Ok(source) = LogSourceRef::new(LogSourceKind::Game, LATEST_LOG) else {
            return tail;
        };
        let at = first_position(&self.location_info.logs_dir(instance_id).join(LATEST_LOG));
        let follower = Follower {
            path: self.location_info.logs_dir(instance_id).join(LATEST_LOG),
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
            match self.announce_new_lines(at).await {
                Some(next) => at = next,
                None => return,
            }
            if stopping {
                return;
            }
        }
    }

    /// Announces the whole lines written since `at`; returns where the next ones begin, or
    /// `None` when the file cannot be followed any more (it was replaced by a shorter one).
    async fn announce_new_lines(&self, at: u64) -> Option<u64> {
        let Ok(page) = read_page(&self.path, LogCursor::After(at), MAX_EVENT_TEXT_BYTES).await
        else {
            // Not there yet, or not readable now: looked at again.
            return Some(at);
        };
        if page.size < at {
            tracing::info!(
                instance_id = %self.instance_id,
                "latest.log was replaced: it is not followed any more"
            );
            return None;
        }

        let whole = whole_lines_len(&page.text);
        if whole == 0 {
            return Some(at);
        }
        let end = if whole == page.text.len() {
            page.end
        } else {
            page.start + whole as u64
        };
        self.emitter
            .emit_safe(GameLogEvent {
                instance_id: self.instance_id.clone(),
                source_id: self.source_id.clone(),
                start: page.start,
                end,
                text: Arc::from(&page.text[..whole]),
            })
            .await;
        Some(end)
    }
}

/// Where the lines to announce begin: the line the file ends in, so that a line the game has
/// half written is announced whole. Read before the following is handed back, so that a line
/// written right after is not missed.
fn first_position(path: &Path) -> u64 {
    let read = || -> std::io::Result<u64> {
        let mut file = std::fs::File::open(path)?;
        let size = file.metadata()?.len();
        let start = size.saturating_sub(START_LOOKBACK_BYTES);
        let mut buf = vec![0u8; usize::try_from(size - start).unwrap_or(0)];
        file.seek(SeekFrom::Start(start))?;
        file.read_exact(&mut buf)?;
        Ok(match buf.iter().rposition(|&b| b == b'\n') {
            Some(pos) => start + pos as u64 + 1,
            None if start > 0 => size,
            None => 0,
        })
    };
    read().unwrap_or(0)
}

/// Bytes of `text` up to and with its last line break; a line still being written is left out.
fn whole_lines_len(text: &str) -> usize {
    text.rfind('\n').map_or(0, |pos| pos + 1)
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
        let tail = rig.tail.follow(&rig.id);

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
        let tail = rig.tail.follow(&rig.id);

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
        let tail = rig.tail.follow(&rig.id);

        append(&rig.log, "last words\n");
        tail.finish(Duration::from_secs(5)).await;

        let event = next_event(&mut rig.events).await;
        assert_eq!(&*event.text, "last words\n");
    }

    #[tokio::test]
    async fn should_follow_a_log_that_appears_after_the_following_began() {
        let mut rig = rig("x\n");
        std::fs::remove_file(&rig.log).unwrap();
        let tail = rig.tail.follow(&rig.id);

        std::fs::write(&rig.log, "fresh\n").unwrap();
        let event = next_event(&mut rig.events).await;

        assert_eq!(&*event.text, "fresh\n");
        tail.finish(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn should_stop_following_when_the_log_is_replaced_by_a_shorter_one() {
        let mut rig = rig("a long first line\n");
        let tail = rig.tail.follow(&rig.id);

        std::fs::write(&rig.log, "short\n").unwrap();
        tail.finish(Duration::from_secs(5)).await;

        assert!(rig.events.try_recv().is_err());
    }
}
