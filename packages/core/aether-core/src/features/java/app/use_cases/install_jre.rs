use std::{path::PathBuf, sync::Arc};

use crate::features::java::JreProvider;

use super::super::JavaApplicationError;

pub struct InstallJreUseCase<JP: JreProvider> {
    provider: Arc<JP>,
}

impl<JP: JreProvider> InstallJreUseCase<JP> {
    pub fn new(provider: Arc<JP>) -> Self {
        Self { provider }
    }

    pub async fn execute(
        &self,
        version: u32,
        install_dir: PathBuf,
    ) -> Result<PathBuf, JavaApplicationError> {
        Ok(self.provider.install(version, &install_dir).await?)
    }
}
