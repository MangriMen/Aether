use std::sync::Arc;

use crate::features::events::app::ports::ListProgressBarsUseCasePort;
use crate::features::events::{Event, EventEmitter, ProgressService};

/// Extension trait providing access to all events feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// events-specific functionality in a centralized manner.
pub trait EventsFeature {
    fn list_progress_bars_use_case(&self) -> Arc<dyn ListProgressBarsUseCasePort>;
    fn event_emitter(&self) -> Arc<dyn EventEmitter<Event>>;
    fn progress_service(&self) -> Arc<dyn ProgressService>;
}
