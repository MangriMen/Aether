use aether_core::features::events::{EventEmitter, EventError};
use async_trait::async_trait;
use tauri::{Emitter, Listener};

pub struct TauriEventEmitter {
    app_handle: tauri::AppHandle,
}

impl TauriEventEmitter {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl EventEmitter for TauriEventEmitter {
    async fn emit_raw(&self, event: &str, payload: serde_json::Value) -> Result<(), EventError> {
        self.app_handle
            .emit(event, payload)
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))
    }

    fn listen_raw(&self, event: String, handler: Box<dyn Fn(String) + Send + 'static>) {
        self.app_handle.listen(event, move |e| {
            let handler = &handler;
            handler(e.payload().to_owned())
        });
    }
}
