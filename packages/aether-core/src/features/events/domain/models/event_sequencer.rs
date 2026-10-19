use log::error;
use tokio::sync::{Mutex, MutexGuard};

use crate::features::events::SharedEventEmitter;

use super::{EventError, ProgressEvent};

/// Sends progress events in the order the changes that made them were made, whoever sends them:
/// the progress service for the changes of a running bar, and the handle of a bar for its end.
/// Without it an event made earlier could be sent after the end of its bar, and bring it back.
///
/// A change is made under the guard of [`begin_change`](Self::begin_change); the guard is
/// handed to [`emit_after`](Self::emit_after) with the events the change made, which are sent
/// after those of every change before it, without holding up the changes after it.
pub(crate) struct EventSequencer {
    emitter: SharedEventEmitter,
    /// Keeps the changes that span bars whole and in order. Never held while events are sent.
    update_lock: Mutex<()>,
    /// Taken before the update lock is let go, so no event of a bar overtakes an earlier one.
    emit_lock: Mutex<()>,
}

impl EventSequencer {
    pub(crate) fn new(emitter: SharedEventEmitter) -> Self {
        Self {
            emitter,
            update_lock: Mutex::new(()),
            emit_lock: Mutex::new(()),
        }
    }

    /// Wait for the turn to change the bars.
    pub(crate) async fn begin_change(&self) -> MutexGuard<'_, ()> {
        self.update_lock.lock().await
    }

    /// Send `events`, made under the change `guard`, after those of every change made before
    /// them, stopping at the first that fails.
    pub(crate) async fn emit_after(
        &self,
        guard: MutexGuard<'_, ()>,
        events: Vec<ProgressEvent>,
    ) -> Result<(), EventError> {
        let Some(_emitting) = self.take_turn(guard, &events).await else {
            return Ok(());
        };
        for event in events {
            self.emitter.emit(event.into()).await?;
        }
        Ok(())
    }

    /// [`emit_after`](Self::emit_after) that goes on past an event that fails, for events that
    /// nobody is left to be told of failing.
    pub(crate) async fn emit_all_after(
        &self,
        guard: MutexGuard<'_, ()>,
        events: Vec<ProgressEvent>,
    ) {
        let Some(_emitting) = self.take_turn(guard, &events).await else {
            return;
        };
        for event in events {
            let (id, status) = (event.progress_bar_id, event.status.clone());
            if let Err(e) = self.emitter.emit(event.into()).await {
                error!("Event {status:?} of progress bar {id} was not delivered: {e:?}");
            }
        }
    }

    /// The turn to send `events` made under the change `guard`, which is let go once it is
    /// taken; none when there is nothing to send.
    async fn take_turn(
        &self,
        guard: MutexGuard<'_, ()>,
        events: &[ProgressEvent],
    ) -> Option<MutexGuard<'_, ()>> {
        if events.is_empty() {
            return None;
        }
        let emitting = self.emit_lock.lock().await;
        drop(guard);
        Some(emitting)
    }
}
