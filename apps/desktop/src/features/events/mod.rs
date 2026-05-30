mod domain;
pub(crate) mod infra;

// Domain models may be re-exported explicitly (Rule 4.4)
pub use domain::AppEvent;
pub use domain::DualEventEmitterExt;

// Facade: explicit re-exports (Rule 4.3)
pub(crate) use infra::get_specta_commands;
pub(crate) use infra::get_specta_events;
pub(crate) use infra::init;
