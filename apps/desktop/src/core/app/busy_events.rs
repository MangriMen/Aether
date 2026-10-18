use std::sync::Arc;

use aether_core::features::events::{CacheEvent, EventEmitterExt, SharedEventEmitter};
use aether_core::shared::housekeeping::ActivityGate;

/// Tells the UI whenever the gate turns busy or idle, so it does not have to ask.
///
/// The task holds only the watch, not the gate, so it ends when the gate is dropped.
pub fn spawn_busy_events(gate: &Arc<ActivityGate>, event_emitter: SharedEventEmitter) {
    let mut watch = gate.watch_busy();
    tokio::spawn(async move {
        while let Some(busy) = watch.changed().await {
            event_emitter
                .emit_safe(CacheEvent::BusyChanged { busy })
                .await;
        }
    });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use aether_core::features::events::{Event, EventEmitter, EventError};
    use tokio::sync::mpsc;

    use super::*;

    struct ChannelEmitter(mpsc::UnboundedSender<Event>);

    #[async_trait::async_trait]
    impl EventEmitter<Event> for ChannelEmitter {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            let _ = self.0.send(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    async fn next_busy(receiver: &mut mpsc::UnboundedReceiver<Event>) -> bool {
        let event = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
            .await
            .expect("an event arrives")
            .expect("the emitter lives");
        match event {
            Event::Cache(CacheEvent::BusyChanged { busy }) => busy,
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_emit_when_the_gate_turns_busy_and_idle() {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let gate = Arc::new(ActivityGate::new());
        spawn_busy_events(&gate, Arc::new(ChannelEmitter(sender)));

        let lease = gate.lease().await;
        assert!(next_busy(&mut receiver).await);

        drop(lease);
        assert!(!next_busy(&mut receiver).await);
    }
}
