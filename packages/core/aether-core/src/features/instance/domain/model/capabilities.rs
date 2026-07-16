use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    /// Identifier for the capability (lowercase, kebab/underscore allowed).
    pub id: String,

    /// Display name of the capability.
    pub name: String,

    /// Optional detailed description of what this capability does.
    pub description: Option<String>,

    /// Optional icon file name or URL for the UI.
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContentProviderCapabilityMetadata {
    pub base: CapabilityMetadata,

    /// Whether the provider supports installing individual items (e.g., a single mod or resource pack).
    pub supports_install_atomic: bool,

    /// Whether the provider supports installing complex modpacks or curated collections.
    pub supports_install_modpacks: bool,
}

impl Deref for ContentProviderCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
