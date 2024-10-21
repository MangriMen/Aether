use std::sync::Arc;

use event_emitter_rs::EventEmitter;

#[derive(Default)]
pub struct AppStateInner {
    pub event_emitter: Arc<tokio::sync::Mutex<EventEmitter>>,
}

pub type AppState = tokio::sync::Mutex<AppStateInner>;
