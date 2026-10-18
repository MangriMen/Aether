use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    java::{Java, JavaApplicationError, app::InstallJava},
};

#[async_trait]
pub trait JavaInstallService: Send + Sync {
    /// Install the Java of `install_java`, unless it is installed already. One download of a
    /// version runs at a time, under a shared bar of its own; an install asking for a version
    /// being downloaded waits for that download.
    ///
    /// `waiting_bar` is the bar of whatever needs the Java: while the install waits on its
    /// download (its own or another's), the bar has a mirror of the download's bar. It gets none
    /// when the Java is there already.
    async fn execute(
        &self,
        install_java: InstallJava,
        waiting_bar: Option<&ProgressBarId>,
    ) -> Result<Java, JavaApplicationError>;
}
