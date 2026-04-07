use crate::features::instance::CapabilityMetadata;

pub trait AsCapabilityMetadata {
    fn as_metadata(&self) -> &CapabilityMetadata;
}
