use std::path::PathBuf;

use crate::shared::CacheId;

pub trait CachePathResolver: Send + Sync {
    fn resolve(&self, namespace: &'static str, id: &CacheId) -> Option<PathBuf>;
}
