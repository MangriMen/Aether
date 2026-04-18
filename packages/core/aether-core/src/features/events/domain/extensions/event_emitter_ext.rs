use async_trait::async_trait;
use tracing::warn;

use crate::features::events::EventEmitter;

#[async_trait]
pub trait EventEmitterExt<E>: EventEmitter<E> {
    async fn emit_safe(&self, event: impl Into<E> + Send) {
        if let Err(err) = self.emit(event.into()).await {
            warn!("Failed to emit event: {err}")
        }
    }

    fn on<T, F>(&self, handler: F)
    where
        E: Send + 'static,
        T: Send + 'static,
        E: TryInto<T>,
        F: Fn(T) + Send + Sync + 'static,
    {
        self.listen(Box::new(move |full_event| {
            if let Ok(specific) = full_event.try_into() {
                handler(specific);
            }
        }));
    }
}

#[async_trait]
impl<T: ?Sized + EventEmitter<E>, E> EventEmitterExt<E> for T {}
