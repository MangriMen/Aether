use std::sync::Arc;

use crate::features::process::app::ports::{
    KillProcessUseCasePort, ListProcessMetadataUseCasePort, ManageProcessService,
    ProcessStartService, TrackProcessService, WaitForProcessUseCasePort,
};

/// Extension trait providing access to all process feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// process-specific functionality in a centralized manner.
pub trait ProcessFeature {
    // ── Use cases ──
    fn wait_for_process_use_case(&self) -> Arc<dyn WaitForProcessUseCasePort>;
    fn kill_process_use_case(&self) -> Arc<dyn KillProcessUseCasePort>;
    fn list_process_metadata_use_case(&self) -> Arc<dyn ListProcessMetadataUseCasePort>;

    // ── Ports (implemented by use cases) ──
    fn process_start_service(&self) -> Arc<dyn ProcessStartService>;
    fn track_process_service(&self) -> Arc<dyn TrackProcessService>;
    fn manage_process_service(&self) -> Arc<dyn ManageProcessService>;
}
