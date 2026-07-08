use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;

use crate::features::java::{
    JavaInstallationTracker, app::ports::GetActiveJavaInstallationsUseCasePort,
};

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
        GetActiveJavaInstallationsUseCasePort::execute(self).await
    }
}

#[async_trait]
impl GetActiveJavaInstallationsUseCasePort for GetActiveJavaInstallationsUseCase {
    async fn execute(&self) -> HashSet<u32> {
        self.java_installation_tracker.get_active_installations()
    }
}
