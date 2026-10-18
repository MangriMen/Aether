use std::{future::Future, sync::Arc};

use log::error;
use tokio::sync::mpsc;

/// Returns a sink that hands events to one consumer task, which runs `handler` on them one at
/// a time in the order they were sent.
///
/// Spawning a task per event lets two quick events finish out of order, so a handler that reads
/// current state can undo what the later event already did.
///
/// Each call runs in its own task that is awaited before the next event starts, so a panic in
/// the handler loses that event only and the consumer keeps going.
pub fn ordered_sink<E, F, Fut>(handler: F) -> impl Fn(E) + Send + Sync + 'static
where
    E: Send + 'static,
    F: Fn(E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (sender, mut receiver) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let handler = Arc::new(handler);
        while let Some(event) = receiver.recv().await {
            let handler = handler.clone();
            if let Err(error) = tokio::spawn(async move { handler(event).await }).await {
                error!("Event handler failed: {error}");
            }
        }
    });

    move |event| {
        // The consumer lives as long as the runtime; a failed send means it is shutting down.
        let _ = sender.send(event);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::{Mutex, mpsc};

    use super::*;

    #[tokio::test]
    async fn should_handle_events_in_the_order_they_were_sent() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (done, mut finished) = mpsc::unbounded_channel();
        let sink = ordered_sink({
            let seen = seen.clone();
            move |event: u32| {
                let seen = seen.clone();
                let done = done.clone();
                async move {
                    // The first event is the slowest: unordered tasks would finish it last.
                    tokio::time::sleep(Duration::from_millis(u64::from(30 - event * 10))).await;
                    seen.lock().await.push(event);
                    let _ = done.send(());
                }
            }
        });

        for event in 0..3 {
            sink(event);
        }
        for _ in 0..3 {
            tokio::time::timeout(Duration::from_secs(5), finished.recv())
                .await
                .expect("the events are handled");
        }

        assert_eq!(*seen.lock().await, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn should_keep_handling_events_after_a_handler_panics() {
        let (done, mut finished) = mpsc::unbounded_channel();
        let sink = ordered_sink(move |event: u32| {
            let done = done.clone();
            async move {
                assert_ne!(event, 0, "the first event panics");
                let _ = done.send(event);
            }
        });

        sink(0);
        sink(1);

        let handled = tokio::time::timeout(Duration::from_secs(5), finished.recv())
            .await
            .expect("the later event is handled");
        assert_eq!(handled, Some(1));
    }
}
