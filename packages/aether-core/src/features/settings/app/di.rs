use std::sync::Arc;

use crate::features::settings::app::ports::{
    ClearCacheUseCasePort, EditDefaultInstanceSettingsUseCasePort, EditSettingsUseCasePort,
    GetCacheUsageUseCasePort, GetDefaultInstanceSettingsUseCasePort, GetSettingsUseCasePort,
    GetStorageUsageUseCasePort,
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
    fn clear_cache_use_case(&self) -> Arc<dyn ClearCacheUseCasePort>;
    fn get_cache_usage_use_case(&self) -> Arc<dyn GetCacheUsageUseCasePort>;
    fn get_storage_usage_use_case(&self) -> Arc<dyn GetStorageUsageUseCasePort>;
    fn edit_settings_use_case(&self) -> Arc<dyn EditSettingsUseCasePort>;
    fn edit_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn EditDefaultInstanceSettingsUseCasePort>;

    // ── Domain services ──
    fn location_info(&self) -> Arc<LocationInfo>;
}
