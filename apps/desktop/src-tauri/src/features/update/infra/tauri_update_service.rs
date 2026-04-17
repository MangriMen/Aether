use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_updater::Update;
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::Mutex;

use crate::features::update::UpdateService;
use crate::features::update::UpdateStatus;

pub struct TauriUpdateService<R: tauri::Runtime> {
    app: AppHandle<R>,
    pending_update: Arc<Mutex<Option<Update>>>,
}

impl<R: tauri::Runtime> TauriUpdateService<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self {
            app,
            pending_update: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl<R: tauri::Runtime> UpdateService for TauriUpdateService<R> {
    async fn check(&self) -> Result<Option<UpdateStatus>, String> {
        let updater = self.app.updater().map_err(|e| e.to_string())?;
        let update = updater.check().await.map_err(|e| e.to_string())?;

        let mut cached_update = self.pending_update.lock().await;
        *cached_update = update.clone();

        Ok(update.map(Into::into))
    }

    async fn install(&self) -> Result<(), String> {
        let mut cache = self.pending_update.lock().await;

        if let Some(update) = cache.take() {
            let mut downloaded = 0;
            let mut content_length: Option<u64> = None;

            update
                .download_and_install(
                    |chunk_length, total_length| {
                        content_length = total_length;
                        downloaded += chunk_length;

                        let fraction = content_length.map(|total| downloaded as f64 / total as f64);

                        let _ = self.app.emit(
                            "loading",
                            json!({
                                "fraction": fraction,
                                "message": "update.progress",
                            }),
                        );
                    },
                    move || {
                        let _ = self.app.emit(
                            "loading",
                            json!({
                                "fraction": "null",
                                "message": "update.finished",
                            }),
                        );
                    },
                )
                .await
                .map_err(|e| e.to_string())?;

            self.app.restart();
        } else {
            Err("No pending update found. Please check for updates first.".to_string())
        }
    }
}

impl From<Update> for UpdateStatus {
    fn from(value: Update) -> Self {
        Self {
            version: Some(value.version),
            date: value.date.map(|d| d.to_string()),
            body: value.body,
        }
    }
}
