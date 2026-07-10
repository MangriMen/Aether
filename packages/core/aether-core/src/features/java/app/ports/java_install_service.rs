use async_trait::async_trait;

use crate::features::java::{Java, JavaApplicationError, app::InstallJava};

#[async_trait]
pub trait JavaInstallService: Send + Sync {
    async fn execute(&self, install_java: InstallJava) -> Result<Java, JavaApplicationError>;
}
