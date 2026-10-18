use std::fmt::Display;

use crate::features::events::ProgressBarId;

/// A bar that may not have been created: the work runs without it then.
pub trait OptionalProgressBarExt {
    /// Mark the bar, when there is one, as failed with the error of `result`, when it is one.
    fn fail_on_err<T, E: Display>(&self, result: &Result<T, E>) -> impl Future<Output = ()> + Send;
}

impl OptionalProgressBarExt for Option<ProgressBarId> {
    fn fail_on_err<T, E: Display>(&self, result: &Result<T, E>) -> impl Future<Output = ()> + Send {
        let failed = self
            .as_ref()
            .zip(result.as_ref().err().map(ToString::to_string));
        async move {
            if let Some((bar, message)) = failed {
                bar.fail(message).await;
            }
        }
    }
}
