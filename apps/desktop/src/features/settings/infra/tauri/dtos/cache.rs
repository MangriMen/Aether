use aether_core::features::settings::{
    CacheKind, CacheUsage, ClearCacheOutcome, StorageKind, StorageUsage,
};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CacheKindDto {
    Metadata,
    Icons,
    ModpackArchives,
}

impl From<CacheKind> for CacheKindDto {
    fn from(value: CacheKind) -> Self {
        match value {
            CacheKind::Metadata => Self::Metadata,
            CacheKind::Icons => Self::Icons,
            CacheKind::ModpackArchives => Self::ModpackArchives,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CacheUsageEntryDto {
    pub kind: CacheKindDto,
    /// A number, not `u64`: specta refuses to export `u64`, and a cache never nears 2^53 bytes.
    pub bytes: f64,
}

/// The bytes as a number the frontend reads: see [`CacheUsageEntryDto::bytes`].
#[allow(clippy::cast_precision_loss)]
fn bytes_to_number(bytes: u64) -> f64 {
    bytes as f64
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CacheUsageDto {
    pub entries: Vec<CacheUsageEntryDto>,
}

impl From<CacheUsage> for CacheUsageDto {
    fn from(value: CacheUsage) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|entry| CacheUsageEntryDto {
                    kind: entry.kind.into(),
                    bytes: bytes_to_number(entry.bytes),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ClearCacheOutcomeDto {
    Cleared,
    Busy,
}

impl From<ClearCacheOutcome> for ClearCacheOutcomeDto {
    fn from(value: ClearCacheOutcome) -> Self {
        match value {
            ClearCacheOutcome::Cleared => Self::Cleared,
            ClearCacheOutcome::Busy => Self::Busy,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum StorageKindDto {
    Minecraft,
    Java,
    Instances,
}

impl From<StorageKind> for StorageKindDto {
    fn from(value: StorageKind) -> Self {
        match value {
            StorageKind::Minecraft => Self::Minecraft,
            StorageKind::Java => Self::Java,
            StorageKind::Instances => Self::Instances,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StorageUsageEntryDto {
    pub kind: StorageKindDto,
    /// A number, not `u64`: see [`CacheUsageEntryDto::bytes`].
    pub bytes: f64,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StorageUsageDto {
    pub entries: Vec<StorageUsageEntryDto>,
}

impl From<StorageUsage> for StorageUsageDto {
    fn from(value: StorageUsage) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|entry| StorageUsageEntryDto {
                    kind: entry.kind.into(),
                    bytes: bytes_to_number(entry.bytes),
                })
                .collect(),
        }
    }
}
