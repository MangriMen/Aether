use serr::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum RegistryError {
    #[error("Capability {capability_type} with id \"{capability_id}\" not found")]
    CapabilityNotFound {
        capability_type: &'static str,
        capability_id: String,
    },
}
