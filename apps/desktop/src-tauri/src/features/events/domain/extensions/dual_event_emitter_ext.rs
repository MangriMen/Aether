use crate::features::events::AppEvent;
use aether_core::features::events::{Event as CoreEvent, EventEmitter, EventEmitterExt};
use std::ops::Deref;

pub trait DualEventEmitterExt {
    fn on_core<T, F>(&self, handler: F)
    where
        T: Send + 'static,
        CoreEvent: TryInto<T>,
        F: Fn(T) + Send + Sync + 'static;

    fn on_app<T, F>(&self, handler: F)
    where
        T: Send + 'static,
        AppEvent: TryInto<T>,
        F: Fn(T) + Send + Sync + 'static;
}

impl<S, Target> DualEventEmitterExt for S
where
    S: Deref<Target = Target>,
    Target: EventEmitter<CoreEvent> + EventEmitter<AppEvent> + ?Sized,
{
    fn on_core<T, F>(&self, handler: F)
    where
        T: Send + 'static,
        CoreEvent: TryInto<T>,
        F: Fn(T) + Send + Sync + 'static,
    {
        EventEmitterExt::<CoreEvent>::on::<T, F>(&**self, handler);
    }

    fn on_app<T, F>(&self, handler: F)
    where
        T: Send + 'static,
        AppEvent: TryInto<T>,
        F: Fn(T) + Send + Sync + 'static,
    {
        EventEmitterExt::<AppEvent>::on::<T, F>(&**self, handler);
    }
}
