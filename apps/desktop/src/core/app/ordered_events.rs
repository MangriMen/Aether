use std::future::Future;

use tokio::sync::mpsc;

/// Returns a sink that hands events to one consumer task, which runs `handler` on them one at
/// a time in the order they were sent.
///
/// Spawning a task per event lets two quick events finish out of order, so a handler that reads
/// current state can undo what the later event already did.
pub fn ordered_sink<E, F, Fut>(handler: F) -> impl Fn(E) + Send + Sync + 'static
where
    E: Send + 'static,
    F: Fn(E) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (sender, mut receiver) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            handler(event).await;
        }
    });

    move |event| {
        // The consumer lives as long as the runtime; a failed send means it is shutting down.
        let _ = sender.send(event);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use tokio::sync::Mutex;

    use super::*;

    #[tokio::test]
    async fn should_handle_events_in_the_order_they_were_sent() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = ordered_sink({
            let seen = seen.clone();
            move |event: u32| {
                let seen = seen.clone();
                async move {
                    // The first event is the slowest: unordered tasks would finish it last.
                    tokio::time::sleep(Duration::from_millis(u64::from(30 - event * 10))).await;
                    seen.lock().await.push(event);
                }
            }
        });

        for event in 0..3 {
            sink(event);
        }
        tokio::time::sleep(Duration::from_millis(150)).await;

        assert_eq!(*seen.lock().await, vec![0, 1, 2]);
    }
}
