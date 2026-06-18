mod app;
mod domain;
pub(crate) mod infra;

// Domain models may be re-exported explicitly (Rule 4.4)
pub use domain::ActionOnInstanceLaunch;
pub use domain::AppSettings;
pub use domain::AppSettingsError;
pub use domain::AppSettingsStorage;
pub use domain::WindowEffect;

// App layer use cases (inbound ports)
pub use app::EditAppSettingsRequest;
pub use app::EditAppSettingsUseCase;
pub use app::GetAppSettingsUseCase;

// Facade: explicit re-exports (Rule 4.3)
pub(crate) use infra::get_specta_commands;
pub(crate) use infra::init;
