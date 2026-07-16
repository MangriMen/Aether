use aether_core_plugin_api::v0::PackSourceDto;

use crate::features::instance::PackSource;

impl From<PackSource> for PackSourceDto {
    fn from(value: PackSource) -> Self {
        match value {
            PackSource::LocalFile(path) => Self::LocalFile(path),
            PackSource::RemoteUrl(url) => Self::RemoteUrl(url),
            PackSource::Registry { .. } => Self::RemoteUrl(String::new()),
        }
    }
}
