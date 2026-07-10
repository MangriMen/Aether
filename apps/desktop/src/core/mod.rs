pub mod app;
mod domain;
pub(crate) mod infra;

// Explicit re-exports
pub(crate) use app::InitializePluginsUseCase;
pub(crate) use app::RecreateWindowUseCase;
pub(crate) use domain::WindowManager;
pub(crate) use domain::{Error, Result, WindowError, WindowLabel};
pub use infra::launch_app;

// Internal re-exports (pub(crate))
pub(crate) use infra::FrontendResult;
pub(crate) use infra::OverridableDto;
pub(crate) use infra::PreventExitState;
pub(crate) use infra::TauriWindowManager;
pub(crate) use infra::format_asset_url;
pub(crate) use infra::get_main_window_state_flags;
pub(crate) use infra::get_specta_commands;
pub(crate) use infra::{
    AppSettingsErrorDto, AuthErrorDto, EventErrorDto, FileWatcherErrorDto, InstanceErrorDto,
    JavaErrorDto, MinecraftErrorDto, PluginErrorDto, ProcessErrorDto, RequestErrorDto,
    SettingsErrorDto, WindowErrorDto,
};
pub(crate) use infra::{
    AppSettingsStorageState, ContainerState, EventEmitterState, LocationInfoState,
    UpdateServiceState, WindowManagerState,
};
