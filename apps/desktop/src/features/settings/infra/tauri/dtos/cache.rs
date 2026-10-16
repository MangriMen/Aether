use aether_core::{
    features::settings::{CacheUsage, ClearCacheOutcome},
    shared::housekeeping::CacheKind,
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

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CacheUsageDto {
    pub entries: Vec<CacheUsageEntryDto>,
    /// Whether clearing is refused now: the app is busy with something that uses the caches.
    pub busy: bool,
}

impl From<CacheUsage> for CacheUsageDto {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: CacheUsage) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|entry| CacheUsageEntryDto {
                    kind: entry.kind.into(),
                    bytes: entry.bytes as f64,
                })
                .collect(),
            busy: value.busy,
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
