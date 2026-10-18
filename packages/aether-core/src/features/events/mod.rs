mod app;
mod domain;
pub mod infra;

// App-layer exports
pub use app::EventsFeature;
pub use app::list_progress_bars::ListProgressBarsUseCase;
pub use app::ports::{
    EventEmitter, ListProgressBarsUseCasePort, ProgressBarChange, ProgressBarStorage,
    ProgressService, SharedEventEmitter,
};
pub use app::services::ProgressServiceImpl;

// Domain models — explicitly re-exported for cross-feature use
pub use domain::{
    CacheEvent, CancellableBar, Event, EventEmitterExt, EventError, InstanceEvent,
    InstanceEventType, OptionalProgressBarExt, PluginEvent, ProcessEvent, ProcessEventType,
    ProgressBar, ProgressBarId, ProgressBarStorageError, ProgressEvent, ProgressEventType,
    ProgressServiceExt, ProgressStage, ProgressStatus, SharedProgressWait, WarningEvent,
};

#[cfg(test)]
pub(crate) mod test_support;
