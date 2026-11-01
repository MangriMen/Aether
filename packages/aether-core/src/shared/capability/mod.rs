pub(crate) mod domain;
pub mod infra;

pub use domain::{
    CapabilityEntry, CapabilityId, CapabilityRegistry, InvalidId, PlatformId, PluginId,
    RegistryError, is_valid_id,
};
