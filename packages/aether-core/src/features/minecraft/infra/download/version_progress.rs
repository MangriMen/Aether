use crate::features::events::{
    CancellableBar, ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt,
    ProgressStage,
};

/// The shared bar of the files of one game version, measured in the bytes they are expected
/// to have, with a bar under it per file (the assets share one). A bar under it that is dropped
/// before it is settled ends cancelled.
pub(crate) struct VersionProgress<'a, PS: ProgressService> {
    service: &'a PS,
    /// `None` when the bar could not be created: the files download without progress.
    bar: Option<&'a ProgressBarId>,
}

impl<'a, PS: ProgressService> VersionProgress<'a, PS> {
    pub(crate) fn new(service: &'a PS, bar: Option<&'a ProgressBarId>) -> Self {
        Self { service, bar }
    }

    /// Start the bar of a download under the version's, of `total` steps; none without the
    /// version's bar.
    pub(crate) async fn start(&self, event_type: ProgressEventType, total: f64) -> CancellableBar {
        let Some(parent) = self.bar else {
            return CancellableBar::new(None);
        };
        let bar = self
            .service
            .init_child_progress_safe(parent.id, event_type, total, "Downloading".into())
            .await;
        if let Some(bar) = &bar {
            self.service
                .set_progress_stage_safe(bar, Some(ProgressStage::Downloading))
                .await;
        }
        CancellableBar::new(bar)
    }

    /// Start the bar of the file `file_name`, which moves with its bytes.
    pub(crate) async fn start_file(&self, file_name: &str) -> CancellableBar {
        self.start(
            ProgressEventType::FileDownload {
                file_name: file_name.to_owned(),
            },
            1.0,
        )
        .await
    }

    /// `bytes` more are expected on the bar `download`, and on the version's with it.
    pub(crate) async fn add_bytes_total(&self, download: &CancellableBar, bytes: u64) {
        if let Some(bar) = download.bar() {
            self.service.add_progress_bytes_total_safe(bar, bytes).await;
        }
    }

    /// One step of the bar `download` is done, bringing `bytes` in, and `size` of what the
    /// version's bar expects is in place with it.
    pub(crate) async fn add_step(&self, download: &CancellableBar, bytes: u64, size: u64) {
        if let Some(bar) = download.bar() {
            self.service.add_progress_bytes_safe(bar, bytes).await;
            self.service.emit_progress_safe(bar, 1.0, None).await;
        }
        self.add_done(size).await;
    }

    /// `bytes` of what the version's bar expects are in place.
    pub(crate) async fn add_done(&self, bytes: u64) {
        if let Some(bar) = self.bar {
            #[allow(clippy::cast_precision_loss)] // a progress fraction
            self.service
                .emit_progress_safe(bar, bytes as f64, None)
                .await;
        }
    }
}
