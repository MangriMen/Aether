//! Putting an instance back as it was when a run fails halfway.

use std::{collections::HashMap, path::Path};

use super::{FileChange, InstallPlanExecutor, Undo};
use crate::features::instance::{InstanceError, InstanceId, InstanceStorageExt, PackFile};

impl InstallPlanExecutor {
    /// Put an existing instance back as it was before the run: the files moved aside back in
    /// place, the files created gone, the records and the game as they were. Goes on past a
    /// step that fails, so as much as possible is put back; `false` when something was not.
    pub(super) async fn roll_back(
        &self,
        instance_id: &InstanceId,
        records_before: &HashMap<String, PackFile>,
        undo: &Undo,
        err: &InstanceError,
    ) -> bool {
        tracing::info!("Failed to update instance \"{instance_id}\": {err}. Rolling back");
        let mut complete = true;
        let mut failed = |what: &str, err: &InstanceError| {
            tracing::error!("Rolling back \"{instance_id}\": failed to {what}: {err}");
            complete = false;
        };

        for (index, change) in undo.files.iter().enumerate().rev() {
            let restored = match change {
                // Moved out rather than removed: a removal falls back to the `.disabled`
                // variant, which may be the user's file.
                FileChange::Created { install_path } => self
                    .content_file_service
                    .move_content_file_out(
                        instance_id,
                        install_path,
                        &undo.backup_dir.join(format!("created-{index}")),
                    )
                    .await
                    .map(drop),
                FileChange::MovedAside {
                    install_path,
                    backup,
                } => {
                    self.content_file_service
                        .install_content_file(instance_id, install_path, backup)
                        .await
                }
            };
            if let Err(err) = restored {
                failed(&change.restore_description(), &err);
            }
        }

        let (restore, forget): (Vec<&String>, Vec<&String>) = undo
            .records
            .iter()
            .partition(|path| records_before.contains_key(*path));
        if !restore.is_empty() {
            let paths: Vec<String> = restore.iter().map(|path| (*path).clone()).collect();
            let files: Vec<PackFile> = restore
                .iter()
                .map(|path| records_before[*path].clone())
                .collect();
            if let Err(err) = self
                .pack_storage
                .update_pack_file_many(instance_id, &paths, &files)
                .await
            {
                failed("restore the records", &err);
            }
        }
        if !forget.is_empty() {
            let paths: Vec<String> = forget.into_iter().cloned().collect();
            if let Err(err) = self
                .pack_storage
                .remove_pack_file_many(instance_id, &paths)
                .await
            {
                failed("remove the new records", &err);
            }
        }

        if let Some(before) = undo.game_before.clone() {
            let restored = self
                .instance_storage
                .upsert_with(instance_id, move |instance| {
                    instance.game_version = before.game_version;
                    instance.loader = before.loader;
                    instance.loader_version = before.loader_version;
                    Ok(())
                })
                .await;
            if let Err(err) = restored {
                failed("restore the game", &err);
            }
        }

        complete
    }

    /// Move what is at `install_path` aside before it is written over or removed. `false` —
    /// nothing was there.
    pub(super) async fn move_aside(
        &self,
        instance_id: &InstanceId,
        install_path: &str,
        undo: &mut Undo,
    ) -> Result<bool, InstanceError> {
        let backup = undo.backup_dir.join(undo.files.len().to_string());
        let moved = self
            .content_file_service
            .move_content_file_out(instance_id, install_path, &backup)
            .await?;
        if moved {
            undo.files.push(FileChange::MovedAside {
                install_path: install_path.to_owned(),
                backup,
            });
        }
        Ok(moved)
    }

    /// Move a staged file to `install_path`, what was there moved aside first.
    pub(super) async fn put_file(
        &self,
        instance_id: &InstanceId,
        install_path: &str,
        staged: &Path,
        undo: &mut Undo,
    ) -> Result<(), InstanceError> {
        let replaced = self.move_aside(instance_id, install_path, undo).await?;
        self.content_file_service
            .install_content_file(instance_id, install_path, staged)
            .await?;
        if !replaced {
            undo.files.push(FileChange::Created {
                install_path: install_path.to_owned(),
            });
        }
        Ok(())
    }
}
