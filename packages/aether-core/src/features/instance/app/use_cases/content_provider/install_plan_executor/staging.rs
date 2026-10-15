//! The directories of a run in the temp directory, and cleaning up what a run left there.

use std::path::{Path, PathBuf};

/// What the directories of a run in the temp directory are named with: `<prefix><run id>` for
/// the staging one, followed by [`BACKUP_DIR_SUFFIX`] for the backups.
const PLAN_DIR_PREFIX: &str = "install-plan-";
const BACKUP_DIR_SUFFIX: &str = "-backup";

/// The directories of a run in the temp directory, removed when the run's future is dropped
/// before it could remove them itself: a cancelled install leaves no staging behind.
///
/// The backup directory goes only when it is empty. One holding files was dropped in the middle
/// of a run, before a rollback could put them back: they are kept, and their place logged.
pub(super) struct PlanDirs {
    pub(super) staging: PathBuf,
    pub(super) backup_dir: PathBuf,
    pub(super) armed: bool,
}

impl PlanDirs {
    /// The directories of a new run in `temp_dir`, not created yet.
    pub(super) fn new(temp_dir: &Path) -> Self {
        let run_id = uuid::Uuid::new_v4();
        Self {
            staging: temp_dir.join(format!("{PLAN_DIR_PREFIX}{run_id}")),
            backup_dir: temp_dir.join(format!("{PLAN_DIR_PREFIX}{run_id}{BACKUP_DIR_SUFFIX}")),
            armed: true,
        }
    }

    /// The run got through to removing its directories itself.
    pub(super) fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for PlanDirs {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }

        // Blocking, but brief and best effort: a drop cannot wait for the runtime.
        let _ = std::fs::remove_dir_all(&self.staging);
        if std::fs::remove_dir(&self.backup_dir).is_err() && self.backup_dir.exists() {
            tracing::error!(
                "An install was stopped halfway: files it moved aside are kept in \"{}\"",
                self.backup_dir.display()
            );
        }
    }
}

/// Remove what runs of a plan left in `temp_dir` when the application stopped in the middle of
/// them: their staging directories, and their backup directories that are empty. One that
/// holds files is kept and logged — it may be the only copy of a file a run moved aside.
///
/// Meant for startup, before any plan runs: a run going on would lose its directories.
pub async fn remove_plan_leftovers(temp_dir: &Path) {
    let mut entries = match tokio::fs::read_dir(temp_dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
        Err(err) => {
            tracing::warn!(
                "Failed to look for leftovers of installs in \"{}\": {err}",
                temp_dir.display()
            );
            return;
        }
    };

    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(err) => {
                tracing::warn!("Failed to look for leftovers of installs: {err}");
                break;
            }
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(PLAN_DIR_PREFIX)
            || !entry.file_type().await.is_ok_and(|kind| kind.is_dir())
        {
            continue;
        }

        let path = entry.path();
        if name.ends_with(BACKUP_DIR_SUFFIX) {
            if tokio::fs::remove_dir(&path).await.is_err() {
                tracing::warn!(
                    "Files an interrupted install moved aside are kept in \"{}\"",
                    path.display()
                );
            }
        } else if let Err(err) = tokio::fs::remove_dir_all(&path).await {
            tracing::warn!(
                "Failed to remove the leftover staging \"{}\": {err}",
                path.display()
            );
        }
    }
}

/// Remove the staging directory, best effort.
///
/// Retried a few times: a cancelled download may still hold a file open for a moment, and on
/// Windows an open file cannot be removed. A leftover stays in the temp directory, never in the
/// instance.
pub(super) async fn remove_staging(staging: &Path) {
    const ATTEMPTS: u64 = 5;

    for attempt in 1..=ATTEMPTS {
        match crate::shared::io::infra::remove_dir_all(staging).await {
            Ok(()) => return,
            // Never created: nothing was downloaded.
            Err(_) if !staging.exists() => return,
            Err(err) if attempt == ATTEMPTS => {
                tracing::debug!("Failed to remove staging directory: {err}");
            }
            // Back off so the handle that still holds a file open has time to close it.
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(20 * attempt)).await,
        }
    }
}
