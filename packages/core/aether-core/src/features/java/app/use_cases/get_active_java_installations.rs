use std::{collections::HashSet, sync::Arc};

use crate::features::java::JavaInstallationTracker;

pub struct GetActiveJavaInstallationsUseCase {
    java_installation_tracker: Arc<dyn JavaInstallationTracker>,
}

impl GetActiveJavaInstallationsUseCase {
    pub fn new(java_installation_tracker: Arc<dyn JavaInstallationTracker>) -> Self {
        Self {
            java_installation_tracker,
        }
    }

    pub async fn execute(&self) -> HashSet<u32> {
        self.java_installation_tracker.get_active_installations()
    }
}
