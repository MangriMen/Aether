pub(crate) mod app;
pub(crate) mod domain;
pub mod infra;

pub use app::CapabilityRegistry;
pub use domain::{
    CapabilityEntry, CapabilityId, InvalidId, PlatformId, PluginId, ProviderId, ProviderIdError,
    RegistryError, is_valid_id,
};
