use std::{future::Future, sync::Arc};

use futures::TryStream;

use crate::features::events::{ProgressBarId, ProgressService, ProgressServiceExt};

#[derive(Debug)]
pub struct ProgressConfigWithMessage<'a> {
    pub progress_bar_id: &'a ProgressBarId,
    pub total_progress: f64,
    pub progress_message: Option<&'a str>,
}

// A drop in replacement to try_for_each_concurrent that emits progress events as it goes
// Key is the key to use for which progress bar- a ProgressBarId. If None, does nothing
// Total is the total amount of progress that the progress bar should take up by all futures in this (will be split evenly amongst them).
// If message is Some(t) you will overwrite this loading bar's message with a custom one
// futures_count is the number of futures that will be run, which is needed as we allow Iterator to be passed in, which doesn't have a size
#[tracing::instrument(skip(progress_service, stream, f))]
pub async fn try_for_each_concurrent_with_progress<PS, ST, Fut, F, T, E>(
    progress_service: Arc<PS>,
    stream: ST,
    limit: Option<usize>,
    futures_count: usize, // num is in here as we allow Iterator to be passed in, which doesn't have a size
    progress_config: Option<&ProgressConfigWithMessage<'_>>,
    mut f: F,
) -> Result<(), E>
where
    PS: ProgressService,
    ST: TryStream<Ok = T> + futures::TryStreamExt<Error = E>,
    F: FnMut(T) -> Fut + Send,
    Fut: Future<Output = Result<(), ST::Error>> + Send,
    T: Send,
{
    let progress_increment = progress_config
        .as_ref()
        .map(|config| config.total_progress / futures_count.max(1) as f64)
        .unwrap_or(1.0);

    let progress_service = progress_service.as_ref();

    stream
        .try_for_each_concurrent(limit, |item| {
            let action_future = f(item);

            async move {
                action_future.await?;

                if let Some(cfg) = progress_config {
                    progress_service
                        .emit_progress_safe(
                            cfg.progress_bar_id,
                            progress_increment,
                            cfg.progress_message,
                        )
                        .await;
                }

                Ok(())
            }
        })
        .await
}
