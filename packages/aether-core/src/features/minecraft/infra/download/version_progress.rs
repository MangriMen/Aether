use crate::features::events::{
    CancellableBar, ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt,
    ProgressStage,
};

/// The shared bar of the files of one game version, measured in the bytes they are expected
/// to have, with a bar under it per file (the assets share one).
pub(crate) struct VersionProgress<'a, PS: ProgressService> {
    service: &'a PS,
    /// `None` when the bar could not be created: the files download without progress.
    bar: Option<&'a ProgressBarId>,
}

impl<'a, PS: ProgressService> VersionProgress<'a, PS> {
    pub(crate) fn new(service: &'a PS, bar: Option<&'a ProgressBarId>) -> Self {
        Self { service, bar }
    }

    pub(crate) fn service(&self) -> &'a PS {
        self.service
    }

    /// Start the bar of a download under the version's, of `total` steps.
    pub(crate) async fn start(&self, event_type: ProgressEventType, total: f64) -> FileProgress {
        let bar = match self.bar {
            Some(parent) => {
                self.service
                    .init_child_progress_safe(parent.id, event_type, total, "Downloading".into())
                    .await
            }
            None => None,
        };
        if let Some(bar) = &bar {
            self.service
                .set_progress_stage_safe(bar, Some(ProgressStage::Downloading))
                .await;
        }
        FileProgress(CancellableBar::new(bar))
    }

    /// Start the bar of the file `file_name`, which moves with its bytes.
    pub(crate) async fn start_file(&self, file_name: &str) -> FileProgress {
        self.start(
            ProgressEventType::FileDownload {
                file_name: file_name.to_owned(),
            },
            1.0,
        )
        .await
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

/// The bar of one download under a version's; dropped before it is settled, cancelled.
pub(crate) struct FileProgress(CancellableBar);

impl FileProgress {
    /// What the download reports its bytes on, when there is a bar.
    pub(crate) fn bar(&self) -> Option<&ProgressBarId> {
        self.0.bar()
    }

    /// End the bar by `result`: done, or failed with its error.
    pub(crate) async fn end<T, E: std::fmt::Display>(self, result: &Result<T, E>) {
        match result {
            Ok(_) => self.0.done(),
            Err(err) => self.0.fail(err.to_string()).await,
        }
    }
}
