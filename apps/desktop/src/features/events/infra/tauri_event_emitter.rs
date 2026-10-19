use std::sync::{Mutex, PoisonError};

use aether_core::features::events::{Event as CoreEvent, EventEmitter, EventError};
use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::features::events::{AppEvent, infra::AppEventExt, infra::CoreEventExt};

pub struct TauriEventEmitter<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
    core_bus: broadcast::Sender<CoreEvent>,
    app_bus: broadcast::Sender<AppEvent>,
    seq: EventNumbering,
}

/// Numbers the events sent to the webview, whatever their type: the frontend drops an event that
/// does not follow the last it saw, as the webview can hand over one it delivered before. The
/// lock is held while an event is numbered and sent, so the numbers reach the webview in order.
#[derive(Default)]
struct EventNumbering(Mutex<u32>);

impl EventNumbering {
    /// Number an event and send it with `send`.
    fn send<T>(&self, send: impl FnOnce(u32) -> T) -> T {
        let mut last = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        *last = last.wrapping_add(1);
        send(*last)
    }
}

impl<R: tauri::Runtime> TauriEventEmitter<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        let (core_tx, _) = broadcast::channel(1024);
        let (app_tx, _) = broadcast::channel(1024);
        Self {
            app_handle,
            core_bus: core_tx,
            app_bus: app_tx,
            seq: EventNumbering::default(),
        }
    }
}

#[async_trait]
impl<R: tauri::Runtime> EventEmitter<CoreEvent> for TauriEventEmitter<R> {
    async fn emit(&self, event: CoreEvent) -> Result<(), EventError> {
        let _ = self.core_bus.send(event.clone());

        self.seq
            .send(|seq| event.emit_to_tauri(&self.app_handle, seq))
            .map_err(handle_err)
    }

    fn listen(&self, handler: Box<dyn Fn(CoreEvent) + Send + Sync + 'static>) {
        spawn_forwarding(self.core_bus.subscribe(), handler);
    }
}

#[async_trait]
impl<R: tauri::Runtime> EventEmitter<AppEvent> for TauriEventEmitter<R> {
    async fn emit(&self, event: AppEvent) -> Result<(), EventError> {
        let _ = self.app_bus.send(event.clone());

        self.seq
            .send(|seq| event.emit_to_tauri(&self.app_handle, seq))
            .map_err(handle_err)
    }

    fn listen(&self, handler: Box<dyn Fn(AppEvent) + Send + Sync + 'static>) {
        spawn_forwarding(self.app_bus.subscribe(), handler);
    }
}

/// Hand the events of `rx` to `handler` for as long as the bus lives. A listener too slow for the
/// bus misses some events, which is no reason to stop hearing the later ones.
fn spawn_forwarding<E: Clone + Send + 'static>(
    mut rx: broadcast::Receiver<E>,
    handler: Box<dyn Fn(E) + Send + Sync + 'static>,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => handler(event),
                Err(broadcast::error::RecvError::Lagged(missed)) => {
                    log::warn!("An event listener missed {missed} events");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn handle_err(e: tauri::Error) -> EventError {
    EventError::SerializeError(anyhow::Error::from(e))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn numbers_reach_the_webview_in_order_from_any_thread() {
        let numbering = Arc::new(EventNumbering::default());
        let sent = Arc::new(Mutex::new(Vec::new()));

        let threads: Vec<_> = (0..8)
            .map(|_| {
                let (numbering, sent) = (numbering.clone(), sent.clone());
                std::thread::spawn(move || {
                    for _ in 0..500 {
                        numbering.send(|seq| sent.lock().unwrap().push(seq));
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }

        let sent = sent.lock().unwrap();
        assert_eq!(*sent, (1..=4000).collect::<Vec<u32>>());
    }

    #[test]
    fn a_listener_that_lagged_goes_on_hearing_the_later_events() {
        let (bus, rx) = broadcast::channel(2);
        let (heard, heard_rx) = std::sync::mpsc::channel();
        // Five events into a bus of two before the listener runs: it misses the first three.
        for event in 1..=5 {
            bus.send(event).unwrap();
        }

        spawn_forwarding(rx, Box::new(move |event: i32| heard.send(event).unwrap()));
        let wait = || heard_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!([wait(), wait()], [4, 5]);

        bus.send(6).unwrap();
        assert_eq!(wait(), 6);
    }
}
