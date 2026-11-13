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

use aether_core::shared::error::SharedSource;

use crate::features::events::AppEvent;
use crate::features::update::UpdatePhase;
use crate::features::update::UpdateProgress;
use crate::features::update::{UpdateError, UpdateService, UpdateStatus};

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
    async fn check(&self) -> Result<Option<UpdateStatus>, UpdateError> {
        let updater = self.app.updater().map_err(check_error)?;
        let update = updater.check().await.map_err(check_error)?;

        let mut cached_update = self.pending_update.lock().await;
        (*cached_update).clone_from(&update);

        Ok(update.map(Into::into))
    }

    async fn install(&self) -> Result<(), UpdateError> {
        let update = {
            let mut cache = self.pending_update.lock().await;
            cache.take().ok_or(UpdateError::NoPendingUpdate)?
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
                    let current_version = update.current_version.clone();

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
            // Logged by the command that returns it.
            let error = install_error(e);
            self.event_emitter
                .emit_safe(UpdateProgress {
                    fraction: None,
                    phase: UpdatePhase::Error(error.clone()),
                    version: update.version,
                    current_version: update.current_version,
                })
                .await;

            return Err(error);
        }

        self.app.request_restart();

        Ok(())
    }
}

/// The updater's own error as a source: the typed library error it wraps when there is one,
/// so that its category can be told.
fn source_of(err: tauri_plugin_updater::Error) -> SharedSource {
    use tauri_plugin_updater::Error as E;

    match err {
        E::Reqwest(err) => Arc::new(err),
        E::Io(err) => Arc::new(err),
        E::Serialization(err) => Arc::new(err),
        other => Arc::new(other),
    }
}

/// A failure of the network, of the signature, or of the update itself.
fn categorize(
    err: tauri_plugin_updater::Error,
    otherwise: fn(SharedSource) -> UpdateError,
) -> UpdateError {
    use tauri_plugin_updater::Error as E;

    match err {
        E::Reqwest(_)
        | E::Network(_)
        | E::Http(_)
        | E::InvalidHeaderValue(_)
        | E::InvalidHeaderName(_) => UpdateError::Network(source_of(err)),
        E::Minisign(_)
        | E::Base64(_)
        | E::SignatureUtf8(_)
        | E::SignedVersionMismatch { .. }
        | E::MissingSignedVersion => UpdateError::Signature(source_of(err)),
        other => otherwise(source_of(other)),
    }
}

/// An error of looking for an update, or of getting the updater at all.
fn check_error(err: tauri_plugin_updater::Error) -> UpdateError {
    categorize(err, UpdateError::Other)
}

/// An error of downloading and installing an update.
fn install_error(err: tauri_plugin_updater::Error) -> UpdateError {
    categorize(err, UpdateError::Install)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tell_a_network_failure_from_a_failed_install() {
        assert!(matches!(
            install_error(tauri_plugin_updater::Error::Network("reset".to_owned())),
            UpdateError::Network(_)
        ));
        assert!(matches!(
            install_error(tauri_plugin_updater::Error::BinaryNotFoundInArchive),
            UpdateError::Install(_)
        ));
    }

    #[test]
    fn should_report_an_invalid_signature_as_such() {
        assert!(matches!(
            install_error(tauri_plugin_updater::Error::SignatureUtf8("x".to_owned())),
            UpdateError::Signature(_)
        ));
    }

    #[test]
    fn should_report_a_failed_check_that_is_not_the_network_as_other() {
        assert!(matches!(
            check_error(tauri_plugin_updater::Error::ReleaseNotFound),
            UpdateError::Other(_)
        ));
    }

    #[test]
    fn should_keep_the_io_error_of_the_updater_as_the_source() {
        let UpdateError::Install(source) = install_error(tauri_plugin_updater::Error::Io(
            std::io::Error::from(std::io::ErrorKind::StorageFull),
        )) else {
            panic!("expected an install error");
        };

        assert!(source.downcast_ref::<std::io::Error>().is_some());
    }
}
