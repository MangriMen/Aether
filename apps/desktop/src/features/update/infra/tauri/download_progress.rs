use std::future::Future;

use aether_core::features::events::{EventEmitterExt, SharedEventEmitter};
use tokio::sync::watch;
use uuid::Uuid;

use crate::features::events::AppEvent;
use crate::features::update::{UpdatePhase, UpdateProgress};

/// The smallest change of the fraction worth a progress event: 0.5 %, finer than a bar shows.
const MIN_FRACTION_STEP: f64 = 0.005;

/// Receives the chunks of a download and keeps the latest fraction for the one that reports it.
pub(crate) struct ProgressSink {
    fraction: watch::Sender<Option<f64>>,
    downloaded: u64,
}

impl ProgressSink {
    fn new(fraction: watch::Sender<Option<f64>>) -> Self {
        Self {
            fraction,
            downloaded: 0,
        }
    }

    pub(crate) fn chunk(&mut self, chunk_length: usize, total_length: Option<u64>) {
        self.downloaded += chunk_length as u64;

        #[expect(
            clippy::cast_precision_loss,
            reason = "a progress fraction; the loss is far below what is displayed"
        )]
        let fraction = total_length
            .filter(|total| *total > 0)
            .map(|total| (self.downloaded as f64 / total as f64).min(1.0));

        self.fraction.send_if_modified(|shown| {
            is_worth_showing(*shown, fraction) && {
                *shown = fraction;
                true
            }
        });
    }
}

fn is_worth_showing(shown: Option<f64>, new: Option<f64>) -> bool {
    match (shown, new) {
        (Some(shown), Some(new)) => new - shown >= MIN_FRACTION_STEP || (new >= 1.0 && shown < 1.0),
        (None, None) => false,
        _ => true,
    }
}

/// Runs a download and reports its progress as `UpdateProgress` events of one progress bar.
///
/// The events go one at a time, in order, and all of them are sent before this returns. A fast
/// download skips the intermediate fractions instead of queuing them. `Finished` is sent only
/// when the download succeeded, so after everything the download itself checks. The sink must
/// be dropped by the download when it ends, or this never returns.
pub(crate) async fn download_with_progress<T, E, Fut>(
    emitter: &SharedEventEmitter<AppEvent>,
    progress_bar_id: Uuid,
    version: &str,
    current_version: &str,
    download: impl FnOnce(ProgressSink) -> Fut,
) -> Result<T, E>
where
    Fut: Future<Output = Result<T, E>>,
{
    let progress = |fraction, phase| UpdateProgress {
        progress_bar_id,
        fraction,
        phase,
        version: version.to_owned(),
        current_version: current_version.to_owned(),
    };

    let (sender, mut receiver) = watch::channel(Some(0.0));

    let report = async {
        while receiver.changed().await.is_ok() {
            let fraction = *receiver.borrow_and_update();
            emitter
                .emit_safe(progress(fraction, UpdatePhase::Progress))
                .await;
        }
    };

    let (result, ()) = tokio::join!(download(ProgressSink::new(sender)), report);

    if result.is_ok() {
        emitter
            .emit_safe(progress(None, UpdatePhase::Finished))
            .await;
    }

    result
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use aether_core::features::events::{EventEmitter, EventError};
    use async_trait::async_trait;

    use super::*;

    struct RecordingEmitter(Mutex<Vec<UpdateProgress>>);

    #[async_trait]
    impl EventEmitter<AppEvent> for RecordingEmitter {
        async fn emit(&self, event: AppEvent) -> Result<(), EventError> {
            let AppEvent::Update(progress) = event;
            // A slow listener: the download goes on meanwhile.
            tokio::task::yield_now().await;
            self.0.lock().unwrap().push(progress);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(AppEvent) + Send + Sync + 'static>) {}
    }

    async fn run(
        chunks: Vec<usize>,
        total: Option<u64>,
        succeed: bool,
    ) -> (Result<(), ()>, Vec<UpdateProgress>) {
        let recorder = Arc::new(RecordingEmitter(Mutex::new(Vec::new())));
        let emitter: SharedEventEmitter<AppEvent> = recorder.clone();

        let result = tokio::time::timeout(
            Duration::from_secs(5),
            download_with_progress(
                &emitter,
                Uuid::nil(),
                "2.0.0",
                "1.0.0",
                |mut sink| async move {
                    for chunk in chunks {
                        sink.chunk(chunk, total);
                        tokio::task::yield_now().await;
                    }
                    if succeed { Ok(()) } else { Err(()) }
                },
            ),
        )
        .await
        .expect("the download must end once its sink is dropped");

        let events = recorder.0.lock().unwrap().clone();
        (result, events)
    }

    fn fractions(events: &[UpdateProgress]) -> Vec<f64> {
        events
            .iter()
            .filter(|e| matches!(e.phase, UpdatePhase::Progress))
            .filter_map(|e| e.fraction)
            .collect()
    }

    #[tokio::test]
    async fn should_send_the_last_progress_before_returning_and_finish_after_it() {
        let (result, events) = run(vec![10; 10], Some(100), true).await;

        assert!(result.is_ok());
        let last_progress = events
            .iter()
            .rposition(|e| matches!(e.phase, UpdatePhase::Progress))
            .unwrap();
        assert_eq!(events[last_progress].fraction, Some(1.0));
        assert_eq!(last_progress, events.len() - 2);
        assert!(matches!(
            events.last().unwrap().phase,
            UpdatePhase::Finished
        ));
    }

    #[tokio::test]
    async fn should_send_progress_in_order_when_chunks_arrive_quickly() {
        let (_, events) = run(vec![1; 1000], Some(1000), true).await;

        let fractions = fractions(&events);
        assert!(fractions.len() > 2);
        assert!(fractions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(*fractions.last().unwrap(), 1.0);
    }

    #[tokio::test]
    async fn should_skip_progress_that_does_not_change_the_shown_fraction() {
        let (_, events) = run(vec![1; 1000], Some(1_000_000), true).await;

        assert!(fractions(&events).is_empty());
    }

    #[tokio::test]
    async fn should_send_the_final_fraction_even_when_the_last_step_is_tiny() {
        let (_, events) = run(vec![996, 4], Some(1000), true).await;

        assert_eq!(*fractions(&events).last().unwrap(), 1.0);
    }

    #[tokio::test]
    async fn should_not_finish_when_the_download_fails() {
        let (result, events) = run(vec![50], Some(100), false).await;

        assert!(result.is_err());
        assert!(
            !events
                .iter()
                .any(|e| matches!(e.phase, UpdatePhase::Finished))
        );
    }

    #[tokio::test]
    async fn should_report_progress_once_when_the_total_is_unknown() {
        let (_, events) = run(vec![10; 20], None, true).await;

        let unknown = events
            .iter()
            .filter(|e| matches!(e.phase, UpdatePhase::Progress) && e.fraction.is_none())
            .count();
        assert_eq!(unknown, 1);
    }
}
