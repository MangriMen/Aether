mod app;
mod domain;
pub mod infra;

// App-layer exports
pub use app::cancel_progress::CancelProgressUseCase;
pub use app::list_progress_bars::ListProgressBarsUseCase;
pub use app::ports::{
    CancelProgressUseCasePort, EventEmitter, ListProgressBarsUseCasePort, ProgressBarChange,
    ProgressBarStorage, ProgressService, SharedEventEmitter,
};
pub use app::services::ProgressServiceImpl;
pub use app::{EventEmitterExt, EventsFeature, ProgressServiceExt};

// Domain models — explicitly re-exported for cross-feature use
pub use domain::{
    CacheEvent, CancelState, CancellableBar, CommandApprovalOutcome, Event, EventError,
    GameLogEvent, InstanceEvent, InstanceEventType, PluginEvent, ProcessEvent, ProcessEventType,
    ProgressBar, ProgressBarId, ProgressBarStorageError, ProgressEvent, ProgressEventType,
    ProgressFailure, ProgressStage, ProgressStatus, SharedDownload, SharedProgressWait,
    WarningEvent,
};

#[cfg(test)]
pub(crate) mod test_support;
