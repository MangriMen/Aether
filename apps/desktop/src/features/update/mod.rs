mod app;
mod domain;
pub(crate) mod infra;

// Domain models may be re-exported explicitly (Rule 4.4)
pub use domain::UpdatePhase;
pub use domain::UpdateProgress;
pub use domain::UpdateService;
pub use domain::UpdateStatus;

// App layer use cases (inbound ports)
pub use app::CheckForUpdatesUseCase;
pub use app::InstallUpdateUseCase;

// Facade: explicit re-exports (Rule 4.3)
pub(crate) use infra::get_specta_commands;
pub(crate) use infra::init;
