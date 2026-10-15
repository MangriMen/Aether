use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::features::java::{JavaInstallationGuard, JavaInstallationTracker};

#[derive(Default)]
pub struct MemoryJavaInstallationTracker {
    locks: Mutex<HashMap<u32, Arc<tokio::sync::Mutex<()>>>>,
}

impl MemoryJavaInstallationTracker {
    fn get_or_create_lock(&self, version: u32) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self.locks.lock().unwrap();
        locks
            .entry(version)
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }
}

#[async_trait]
impl JavaInstallationTracker for MemoryJavaInstallationTracker {
    async fn lock(&self, version: u32) -> JavaInstallationGuard {
        let version_lock = self.get_or_create_lock(version);

        let raw_guard = version_lock.lock_owned().await;

        JavaInstallationGuard::new(move || {
            drop(raw_guard);
        })
    }

    fn try_lock(&self, version: u32) -> Option<JavaInstallationGuard> {
        let version_lock = self.get_or_create_lock(version);

        version_lock.try_lock_owned().ok().map(|raw_guard| {
            JavaInstallationGuard::new(move || {
                drop(raw_guard);
            })
        })
    }

    fn get_active_installations(&self) -> HashSet<u32> {
        let locks = self.locks.lock().unwrap();
        locks
            .iter()
            .filter(|(_, async_lock)| async_lock.try_lock().is_err())
            .map(|(&version, _)| version)
            .collect()
    }
}
