mod app_event;

use std::sync::Arc;

pub use app_event::*;

use crate::features::events::TauriEventEmitter;

pub type EventEmitterState<R> = Arc<TauriEventEmitter<R>>;
