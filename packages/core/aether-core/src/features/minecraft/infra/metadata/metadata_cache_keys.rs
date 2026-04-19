use crate::{
    features::minecraft::{ModLoader, modded, vanilla},
    shared::{CacheId, CacheKey, CachedValue},
};

pub enum MinecraftMetadataCacheNamespaces {
    VersionManifest,
    LoaderManifest,
}

impl MinecraftMetadataCacheNamespaces {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::VersionManifest => "minecraft:version-manifest",
            Self::LoaderManifest => "minecraft:loader-manifest",
        }
    }
}

pub fn version_manifest_key() -> CacheKey<CachedValue<vanilla::VersionManifest>> {
    CacheKey::new(
        MinecraftMetadataCacheNamespaces::VersionManifest.as_str(),
        CacheId::Static("version-manifest"),
    )
}

pub fn loader_manifest_key(loader: ModLoader) -> CacheKey<CachedValue<modded::Manifest>> {
    CacheKey::new(
        MinecraftMetadataCacheNamespaces::LoaderManifest.as_str(),
        CacheId::Named(loader.as_meta_str().to_string()),
    )
}
