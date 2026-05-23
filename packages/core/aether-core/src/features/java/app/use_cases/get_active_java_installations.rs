use std::{collections::HashSet, sync::Arc};

use crate::features::java::JavaInstallationTracker;

pub struct GetActiveJavaInstallationsUseCase<JIT: JavaInstallationTracker> {
    java_installation_tracker: Arc<JIT>,
}

impl<JIT: JavaInstallationTracker> GetActiveJavaInstallationsUseCase<JIT> {
    pub fn new(java_installation_tracker: Arc<JIT>) -> Self {
        Self {
            java_installation_tracker,
        }
    }

    pub async fn execute(&self) -> HashSet<u32> {
        self.java_installation_tracker.get_active_installations()
    }
}
