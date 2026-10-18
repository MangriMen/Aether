//! The bar that bars started deep inside a call belong under, carried with the task rather than
//! through every port on the way: installing a pack into a new instance installs its game and
//! Java through use cases that know nothing of the pack's bar.

use std::future::Future;

use uuid::Uuid;

use super::ProgressBarId;

tokio::task_local! {
    static PROGRESS_PARENT: Uuid;
}

/// Run `future` with the bars it starts through
/// [`init_progress_in_scope_safe`](crate::features::events::ProgressServiceExt::init_progress_in_scope_safe)
/// placed under `parent`; with no parent, as it is.
///
/// The parent goes with the task: a task spawned inside `future` starts its bars at the top.
pub async fn with_progress_parent<F: Future>(
    parent: Option<&ProgressBarId>,
    future: F,
) -> F::Output {
    match parent {
        Some(parent) => PROGRESS_PARENT.scope(parent.id, future).await,
        None => future.await,
    }
}

/// The parent of the closest [`with_progress_parent`] around the current task, if any.
pub(crate) fn current_progress_parent() -> Option<Uuid> {
    PROGRESS_PARENT.try_with(|parent| *parent).ok()
}
