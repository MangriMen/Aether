mod app;
mod domain;
pub mod infra;

// App-layer exports
pub use app::list_progress_bars::ListProgressBarsUseCase;
pub use app::ports::{EventEmitter, ProgressBarStorage, ProgressService, SharedEventEmitter};
pub use app::services::ProgressServiceImpl;
pub use app::stream_utils::{ProgressConfigWithMessage, try_for_each_concurrent_with_progress};

// Domain models — explicitly re-exported for cross-feature use
pub use domain::{
    Event, EventEmitterExt, EventError, InstanceEvent, InstanceEventType, PluginEvent,
    ProcessEvent, ProcessEventType, ProgressBar, ProgressBarId, ProgressBarStorageError,
    ProgressBarStorageExt, ProgressConfig, ProgressEvent, ProgressEventType, ProgressServiceExt,
    WarningEvent,
};
