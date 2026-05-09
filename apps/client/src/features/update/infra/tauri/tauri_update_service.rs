use std::sync::Arc;

use aether_core::features::events::EventEmitterExt;
use aether_core::features::events::SharedEventEmitter;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::TimeZone;
use tauri::AppHandle;
use tauri::webview::cookie::time::OffsetDateTime;
use tauri_plugin_updater::Update;
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::Mutex;

use crate::features::events::AppEvent;
use crate::features::update::UpdatePhase;
use crate::features::update::UpdateProgress;
use crate::features::update::{UpdateService, UpdateStatus};

pub struct TauriUpdateService<R: tauri::Runtime> {
    app: AppHandle<R>,
    event_emitter: SharedEventEmitter<AppEvent>,
    pending_update: Arc<Mutex<Option<Update>>>,
}

impl<R: tauri::Runtime> TauriUpdateService<R> {
    pub fn new(app: AppHandle<R>, event_emitter: SharedEventEmitter<AppEvent>) -> Self {
        Self {
            app,
            event_emitter,
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
        (*cached_update).clone_from(&update);

        Ok(update.map(Into::into))
    }

    async fn install(&self) -> Result<(), String> {
        let update = {
            let mut cache = self.pending_update.lock().await;
            cache
                .take()
                .ok_or("No pending update found. Please check for updates first.")?
        };

        self.event_emitter
            .emit_safe(UpdateProgress {
                fraction: Some(0.0),
                phase: UpdatePhase::Started,
                version: update.version.clone(),
                current_version: update.current_version.clone(),
            })
            .await;

        let mut downloaded = 0;
        let mut content_length: Option<u64> = None;

        let version = update.version.clone();
        let current_version = update.current_version.clone();

        let update_result = update
            .download_and_install(
                |chunk_length, total_length| {
                    downloaded += chunk_length;
                    content_length = total_length;

                    #[allow(clippy::cast_precision_loss)]
                    let fraction = content_length.map(|total| downloaded as f64 / total as f64);

                    let version = update.version.clone();
                    let current_version = update.version.clone();

                    let emitter = self.event_emitter.clone();
                    tokio::spawn(async move {
                        emitter
                            .emit_safe(UpdateProgress {
                                fraction,
                                phase: UpdatePhase::Progress,
                                version,
                                current_version,
                            })
                            .await;
                    });
                },
                move || {
                    let version = version.clone();
                    let current_version = current_version.clone();

                    let emitter = self.event_emitter.clone();
                    tokio::spawn(async move {
                        emitter
                            .emit_safe(UpdateProgress {
                                fraction: None,
                                phase: UpdatePhase::Finished,
                                version,
                                current_version,
                            })
                            .await;
                    });
                },
            )
            .await;

        if let Err(e) = update_result {
            self.event_emitter
                .emit_safe(UpdateProgress {
                    fraction: None,
                    phase: UpdatePhase::Error,
                    version: update.version,
                    current_version: update.current_version,
                })
                .await;

            return Err(e.to_string());
        }

        self.app.request_restart();

        Ok(())
    }
}

impl From<Update> for UpdateStatus {
    fn from(value: Update) -> Self {
        Self {
            version: Some(value.version),
            date: value.date.map(time_to_chrono_fixed),
            body: value.body,
        }
    }
}

pub fn time_to_chrono_fixed(t: OffsetDateTime) -> DateTime<FixedOffset> {
    let timestamp = t.unix_timestamp();
    let nanoseconds = t.nanosecond();

    // Get offset in seconds from 'time'
    let offset_seconds = t.offset().whole_seconds();

    // Create 'chrono' offset
    let offset = FixedOffset::east_opt(offset_seconds).expect("Invalid offset");

    // Combine into DateTime<FixedOffset>
    offset.timestamp_opt(timestamp, nanoseconds).unwrap()
}
