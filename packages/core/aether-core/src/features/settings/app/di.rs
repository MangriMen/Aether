use std::sync::Arc;

use crate::features::settings::app::ports::{
    DefaultInstanceSettingsStorage, EditDefaultInstanceSettingsUseCasePort,
    EditSettingsUseCasePort, GetDefaultInstanceSettingsUseCasePort, GetSettingsUseCasePort,
    SettingsStorage,
};
use crate::features::settings::domain::LocationInfo;

/// Extension trait providing access to all settings feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// settings-specific functionality in a centralized manner.
pub trait SettingsFeature {
    // ── Use cases ──
    fn get_settings_use_case(&self) -> Arc<dyn GetSettingsUseCasePort>;
    fn get_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn GetDefaultInstanceSettingsUseCasePort>;
    fn edit_settings_use_case(&self) -> Arc<dyn EditSettingsUseCasePort>;
    fn edit_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn EditDefaultInstanceSettingsUseCasePort>;

    // ── Ports ──
    fn settings_storage(&self) -> Arc<dyn SettingsStorage>;
    fn default_instance_settings_storage(&self) -> Arc<dyn DefaultInstanceSettingsStorage>;

    // ── Domain services ──
    fn location_info(&self) -> Arc<LocationInfo>;
}
