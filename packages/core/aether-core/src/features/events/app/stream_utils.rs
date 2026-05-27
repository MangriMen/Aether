use std::{future::Future, sync::Arc};

use futures::TryStream;

use crate::features::events::{ProgressBarId, ProgressService, ProgressServiceExt};

#[derive(Debug)]
pub struct ProgressConfigWithMessage<'a> {
    pub progress_bar_id: &'a ProgressBarId,
    pub total_progress: f64,
    pub progress_message: Option<&'a str>,
}

#[tracing::instrument(skip(progress_service, stream, f))]
pub async fn try_for_each_concurrent_with_progress<PS, ST, Fut, F, T, E>(
    progress_service: Arc<PS>,
    stream: ST,
    limit: Option<usize>,
    futures_count: usize,
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
    #[allow(clippy::cast_precision_loss)]
    let progress_increment = progress_config.as_ref().map_or(1.0, |config| {
        config.total_progress / futures_count.max(1) as f64
    });

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
