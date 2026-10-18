use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::features::instance::domain::{InstanceError, InstanceId};

/// The suffix a disabled content file carries on disk: `mods/a.jar` is kept as
/// `mods/a.jar.disabled`, its content path staying `mods/a.jar`.
pub(crate) const DISABLED_SUFFIX: &str = ".disabled";

/// A content file as it is on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedContentFile {
    /// The file itself: the content path, or its [`DISABLED_SUFFIX`] variant.
    pub path: PathBuf,
    pub disabled: bool,
}

#[async_trait]
pub trait ContentFileService: Send + Sync {
    /// Removes content files from disk for the given instance.
    /// Also handles `.disabled` variants.
    ///
    /// A file that is already gone counts as removed. The first file that cannot be removed
    /// stops the call with an error: the files before it are gone, it and those after it are
    /// not, so a caller dropping records removes one path at a time.
    ///
    /// Here and in [`Self::enable_content_files`] and [`Self::disable_content_files`], every
    /// path is checked as in [`Self::check_content_path`] before any file is touched, and each
    /// file must resolve inside the instance directory, as in [`Self::locate_content_file`].
    async fn remove_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    /// Renames `.disabled` → original for the given content paths.
    async fn enable_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    /// Renames original → `.disabled` for the given content paths.
    async fn disable_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    /// Where the file of `content_path` is on disk: under that path, or disabled under its
    /// [`DISABLED_SUFFIX`] variant. The enabled file wins when both exist, as in
    /// [`Self::remove_content_files`]. `None` — neither is a file.
    ///
    /// `content_path` is checked as in [`Self::check_content_path`], and the file must resolve
    /// inside the instance directory (no symlink out of it).
    async fn locate_content_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
    ) -> Result<Option<LocatedContentFile>, InstanceError>;

    /// Refuses a content path that [`Self::install_content_file`] would refuse on its rules
    /// alone: anything but a plain relative chain of names, and anything under `.metadata`.
    /// Touches no file, so a caller can check every path before downloading the first one.
    fn check_content_path(&self, content_path: &str) -> Result<(), InstanceError>;

    /// Moves the file at exactly `content_path` (no [`DISABLED_SUFFIX`] variant is looked for)
    /// out of the instance to `destination`, so that [`Self::install_content_file`] can put it
    /// back. `false` — nothing is there; a directory there is an error.
    ///
    /// `content_path` is checked as in [`Self::install_content_file`].
    async fn move_content_file_out(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        destination: &Path,
    ) -> Result<bool, InstanceError>;

    /// Creates parent directories and moves a temp file to the final
    /// content path inside the instance directory.
    ///
    /// `content_path` is checked as in [`Self::check_content_path`] and must resolve inside
    /// the instance directory on disk too (no symlink out of it); otherwise nothing is moved.
    async fn install_content_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        temp_path: &Path,
    ) -> Result<(), InstanceError>;
}
