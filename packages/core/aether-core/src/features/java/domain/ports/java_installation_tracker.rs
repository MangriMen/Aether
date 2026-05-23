use std::collections::HashSet;

use async_trait::async_trait;

use crate::features::java::JavaInstallationGuard;

#[async_trait]
pub trait JavaInstallationTracker: Send + Sync {
    async fn lock(&self, version: u32) -> JavaInstallationGuard;

    fn try_lock(&self, version: u32) -> Option<JavaInstallationGuard>;

    fn get_active_installations(&self) -> HashSet<u32>;
}
