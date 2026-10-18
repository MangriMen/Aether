use crate::{
    features::minecraft::{ModLoader, modded, vanilla},
    shared::cache::domain::{CacheId, CacheKey, CachedValue},
};

pub(super) enum MinecraftMetadataCacheNamespaces {
    VersionManifest,
    LoaderManifest,
}

impl MinecraftMetadataCacheNamespaces {
    pub(super) fn as_str(&self) -> &'static str {
        match *self {
            Self::VersionManifest => "minecraft:version-manifest",
            Self::LoaderManifest => "minecraft:loader-manifest",
        }
    }
}

/// Every namespace the Minecraft metadata is cached in, for the app to let the user clear them.
pub fn minecraft_metadata_namespaces() -> [&'static str; 2] {
    [
        MinecraftMetadataCacheNamespaces::VersionManifest.as_str(),
        MinecraftMetadataCacheNamespaces::LoaderManifest.as_str(),
    ]
}

pub(crate) fn version_manifest_key() -> CacheKey<CachedValue<vanilla::VersionManifest>> {
    CacheKey::new(
        MinecraftMetadataCacheNamespaces::VersionManifest.as_str(),
        CacheId::Static("version-manifest"),
    )
}

pub(crate) fn loader_manifest_key(loader: ModLoader) -> CacheKey<CachedValue<modded::Manifest>> {
    CacheKey::new(
        MinecraftMetadataCacheNamespaces::LoaderManifest.as_str(),
        // The manifest format version is part of the key, so a cached manifest of an older format is
        // never read as the current one.
        CacheId::Named(daedalus::modded::loader_manifest_metadata(loader.as_meta_str()).cache_key),
    )
}
