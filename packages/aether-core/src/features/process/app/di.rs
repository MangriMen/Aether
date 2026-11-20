use std::sync::Arc;

use crate::features::process::app::ports::{
    AdoptGameProcessesUseCasePort, KillProcessUseCasePort, ListProcessMetadataUseCasePort,
    ManageProcessService, ProcessStartService, RunningGameService,
    TakeSkippedHookNoticesUseCasePort, TrackProcessService,
};

/// Extension trait providing access to all process feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// process-specific functionality in a centralized manner.
pub trait ProcessFeature {
    // ── Use cases ──
    fn kill_process_use_case(&self) -> Arc<dyn KillProcessUseCasePort>;
    fn list_process_metadata_use_case(&self) -> Arc<dyn ListProcessMetadataUseCasePort>;
    /// Run once, when the launcher starts.
    fn adopt_game_processes_use_case(&self) -> Arc<dyn AdoptGameProcessesUseCasePort>;
    fn take_skipped_hook_notices_use_case(&self) -> Arc<dyn TakeSkippedHookNoticesUseCasePort>;

    // ── Ports (implemented by use cases) ──
    fn process_start_service(&self) -> Arc<dyn ProcessStartService>;
    fn track_process_service(&self) -> Arc<dyn TrackProcessService>;
    fn manage_process_service(&self) -> Arc<dyn ManageProcessService>;
    fn running_game_service(&self) -> Arc<dyn RunningGameService>;
}
