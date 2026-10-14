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
pub struct ImporterCapabilityMetadata {
    pub base: CapabilityMetadata,

    ///Optional field label shown in the importer UI.
    pub field_label: Option<String>,

    /// List of supported file extensions, e.g., [`zip`, `mrpack`].
    pub supported_extensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UpdaterCapabilityMetadata {
    pub base: CapabilityMetadata,
}

#[derive(Debug, Clone)]
pub struct ContentProviderCapabilityMetadata {
    pub base: CapabilityMetadata,

    /// The platform whose content this provider serves (`modrinth`, `curseforge`), written as
    /// `[a-z0-9][a-z0-9_-]*`. It is what a [`ContentSource::ProviderRef`] names instead of a
    /// provider, so several providers may declare the same platform; the core picks one.
    ///
    /// `None` — the provider owns no content of a platform and cannot be the target of a ref.
    ///
    /// [`ContentSource::ProviderRef`]: super::ContentSource::ProviderRef
    pub platform: Option<String>,
}

impl Deref for ImporterCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for UpdaterCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for ContentProviderCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
