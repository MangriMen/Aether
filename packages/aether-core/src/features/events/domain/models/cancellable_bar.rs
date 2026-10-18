use std::fmt::Display;

use super::ProgressBarId;

/// The bar of a part of some work that the failure of another part may cut short: the parts
/// run side by side, and the first failure drops the rest halfway. Dropped before it is settled
/// ([`done`](Self::done) or [`fail`](Self::fail)), the bar ends cancelled rather than done.
pub struct CancellableBar {
    /// `None` when the bar could not be created: the work runs without it.
    bar: Option<ProgressBarId>,
    settled: bool,
}

impl CancellableBar {
    pub fn new(bar: Option<ProgressBarId>) -> Self {
        Self {
            bar,
            settled: false,
        }
    }

    pub fn bar(&self) -> Option<&ProgressBarId> {
        self.bar.as_ref()
    }

    /// End the bar as done.
    pub fn done(mut self) {
        self.settled = true;
    }

    /// End the bar as failed with `message`.
    pub async fn fail(mut self, message: impl Into<String> + Send) {
        if let Some(bar) = &self.bar {
            bar.fail(message).await;
        }
        self.settled = true;
    }

    /// End the bar by `result`: done, or failed with its error.
    pub async fn settle<T, E: Display>(self, result: &Result<T, E>) {
        match result {
            Ok(_) => self.done(),
            Err(err) => self.fail(err.to_string()).await,
        }
    }
}

impl Drop for CancellableBar {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        if let Some(bar) = self.bar.take() {
            // Marked first, ended when the task drops it.
            tokio::spawn(async move { bar.cancel().await });
        }
    }
}
