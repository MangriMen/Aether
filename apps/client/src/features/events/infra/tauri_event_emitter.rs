use aether_core::features::events::{Event as CoreEvent, EventEmitter, EventError};
use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::features::events::{AppEvent, AppEventExt, CoreEventExt};

pub struct TauriEventEmitter<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
    core_bus: broadcast::Sender<CoreEvent>,
    app_bus: broadcast::Sender<AppEvent>,
}

impl<R: tauri::Runtime> TauriEventEmitter<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        let (core_tx, _) = broadcast::channel(1024);
        let (app_tx, _) = broadcast::channel(1024);
        Self {
            app_handle,
            core_bus: core_tx,
            app_bus: app_tx,
        }
    }
}

#[async_trait]
impl<R: tauri::Runtime> EventEmitter<CoreEvent> for TauriEventEmitter<R> {
    async fn emit(&self, event: CoreEvent) -> Result<(), EventError> {
        let _ = self.core_bus.send(event.clone());

        event.emit_to_tauri(&self.app_handle).map_err(handle_err)
    }

    fn listen(&self, handler: Box<dyn Fn(CoreEvent) + Send + Sync + 'static>) {
        let mut rx = self.core_bus.subscribe();

        tauri::async_runtime::spawn(async move {
            while let Ok(event) = rx.recv().await {
                handler(event);
            }
        });
    }
}

#[async_trait]
impl<R: tauri::Runtime> EventEmitter<AppEvent> for TauriEventEmitter<R> {
    async fn emit(&self, event: AppEvent) -> Result<(), EventError> {
        let _ = self.app_bus.send(event.clone());

        event.emit_to_tauri(&self.app_handle).map_err(handle_err)
    }

    fn listen(&self, handler: Box<dyn Fn(AppEvent) + Send + Sync + 'static>) {
        let mut rx = self.app_bus.subscribe();
        tauri::async_runtime::spawn(async move {
            while let Ok(event) = rx.recv().await {
                handler(event);
            }
        });
    }
}

fn handle_err(e: tauri::Error) -> EventError {
    EventError::SerializeError(anyhow::Error::from(e))
}
