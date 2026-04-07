use crate::shared::{CacheId, CacheKey};

pub enum MinecraftDownloadCacheNamespaces {
    AssetsIndex,
    VersionInfo,
    VersionJar,
}

impl MinecraftDownloadCacheNamespaces {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::AssetsIndex => "minecraft:assets-index",
            Self::VersionInfo => "minecraft:version-info",
            Self::VersionJar => "minecraft:version-jar",
        }
    }
}

pub fn assets_index_key(id: String) -> CacheKey<daedalus::minecraft::AssetsIndex> {
    CacheKey::new(
        MinecraftDownloadCacheNamespaces::AssetsIndex.as_str(),
        CacheId::Named(id),
    )
}

pub fn version_info_key(version_id: String) -> CacheKey<daedalus::minecraft::VersionInfo> {
    CacheKey::new(
        MinecraftDownloadCacheNamespaces::VersionInfo.as_str(),
        CacheId::Named(version_id),
    )
}

pub fn version_jar_key(version_id: String) -> CacheKey<()> {
    CacheKey::new(
        MinecraftDownloadCacheNamespaces::VersionJar.as_str(),
        CacheId::Named(version_id),
    )
}
