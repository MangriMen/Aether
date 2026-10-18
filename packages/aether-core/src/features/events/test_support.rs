//! What the tests of progress share: an emitter into a channel, and ways to read what the real
//! progress service stores and sends.

use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::features::events::{
    Event, EventEmitter, EventError, ProgressBarStorage, ProgressEvent, ProgressEventType,
    infra::InMemoryProgressBarStorage,
};

/// Sends every event into a channel.
pub(crate) struct ChannelEmitter(pub(crate) mpsc::UnboundedSender<Event>);

#[async_trait]
impl EventEmitter<Event> for ChannelEmitter {
    async fn emit(&self, event: Event) -> Result<(), EventError> {
        let _ = self.0.send(event);
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

/// The kind of bar of a pack, which waits on shared downloads.
pub(crate) fn pack(name: &str) -> ProgressEventType {
    ProgressEventType::PluginDownload {
        plugin_name: name.to_owned(),
    }
}

/// The mirrors in `storage` now, as `(parent, mirrored bar)`.
pub(crate) async fn mirrors_now(storage: &InMemoryProgressBarStorage) -> Vec<(Uuid, Uuid)> {
    let mut mirrors: Vec<_> = storage
        .list()
        .await
        .into_iter()
        .filter_map(|bar| Some((bar.parent_id?, bar.mirror_of?)))
        .collect();
    mirrors.sort_unstable();
    mirrors
}

/// The mirrors in `storage`, as `(parent, mirrored bar)`, once there are `count` of them.
pub(crate) async fn mirrors(
    storage: &InMemoryProgressBarStorage,
    count: usize,
) -> Vec<(Uuid, Uuid)> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let mirrors = mirrors_now(storage).await;
            if mirrors.len() >= count {
                return mirrors;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the mirrors never came")
}

/// Every progress event waiting in `events`.
pub(crate) fn sent(events: &mut mpsc::UnboundedReceiver<Event>) -> Vec<ProgressEvent> {
    std::iter::from_fn(|| events.try_recv().ok())
        .filter_map(|event| match event {
            Event::Progress(event) => Some(event),
            _ => None,
        })
        .collect()
}
