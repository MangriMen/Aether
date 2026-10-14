//! Checking a plan as data, before anything is resolved or downloaded.

use std::collections::HashSet;

use super::super::plan_overrides;
use super::{InstallPlanExecutor, PlanOverrides, path_key};
use crate::features::instance::{
    FileSide, InstallFileEntry, InstanceError, OverrideSource, app::DISABLED_SUFFIX,
};

impl InstallPlanExecutor {
    /// A plan of more files than [`ContentInstallLimits::max_plan_files`](crate::shared::limits::ContentInstallLimits::max_plan_files) is refused as it comes,
    /// before anything of it is resolved.
    pub(super) fn check_file_count(&self, count: usize) -> Result<(), InstanceError> {
        if count > self.limits.max_plan_files {
            return Err(InstanceError::ContentProviderError {
                reason: format!(
                    "The plan lists {count} files, more than {}",
                    self.limits.max_plan_files
                ),
            });
        }
        Ok(())
    }

    /// Every path is a valid content path, and no two files claim the same one — the second
    /// would silently replace the first. Compared as [`path_key`]s: on a case-insensitive file
    /// system, paths differing only in case are one file.
    pub(super) fn check_paths(&self, files: &[InstallFileEntry]) -> Result<(), InstanceError> {
        let mut seen = HashSet::new();

        for file in files {
            self.content_file_service
                .check_content_path(&file.relative_path)?;

            // `X` and `X.disabled` are one file, the second the disabled state of the first.
            let key = path_key(&file.relative_path);
            let key = key.strip_suffix(DISABLED_SUFFIX).unwrap_or(&key).to_owned();
            if !seen.insert(key) {
                return Err(InstanceError::ContentDuplication {
                    content_path: file.relative_path.clone(),
                });
            }
        }

        Ok(())
    }

    /// List the overrides of this side and check their paths as the planned ones, leaving out
    /// those a planned file takes the place of — the planned files win — and those a later
    /// source has a file for: sources are layered, the later on top.
    pub(super) async fn check_overrides(
        &self,
        sources: Vec<OverrideSource>,
        files: &[InstallFileEntry],
    ) -> Result<PlanOverrides, InstanceError> {
        let planned: HashSet<String> = files
            .iter()
            .flat_map(|file| {
                let path = path_key(&file.relative_path);
                [format!("{path}{DISABLED_SUFFIX}"), path]
            })
            .collect();

        let limits = self.limits;
        let mut listed_count = 0;
        let mut checked = Vec::with_capacity(sources.len());

        for OverrideSource { location, side } in sources {
            // As for the planned files: the source stays in the plan, the side is ours to apply.
            if side == FileSide::Server {
                tracing::debug!("Override source {location:?} skipped: it is for the server");
                continue;
            }

            let listed =
                plan_overrides::list_overrides(&location, limits.max_override_files - listed_count)
                    .await?;
            listed_count += listed.len();
            if listed_count > limits.max_override_files {
                return Err(InstanceError::ContentProviderError {
                    reason: format!(
                        "The overrides of the plan hold more than {} files",
                        limits.max_override_files
                    ),
                });
            }
            let mut seen = HashSet::new();
            let mut kept = Vec::with_capacity(listed.len());

            for file in listed {
                self.content_file_service
                    .check_content_path(&file.relative_path)?;

                if !seen.insert(file.relative_path.clone()) {
                    return Err(InstanceError::ContentDuplication {
                        content_path: file.relative_path,
                    });
                }

                if planned.contains(&path_key(&file.relative_path)) {
                    tracing::debug!(
                        "Override \"{}\" skipped: the plan installs that path",
                        file.relative_path
                    );
                    continue;
                }

                kept.push(file);
            }

            checked.push((location, kept));
        }

        // Across sources paths are compared as path keys: on a case-insensitive file system
        // they are one file, and the later source's must be the one left.
        let mut claimed_later = HashSet::new();
        for (_, files) in checked.iter_mut().rev() {
            let keys: Vec<String> = files
                .iter()
                .map(|file| path_key(&file.relative_path))
                .collect();
            files.retain(|file| !claimed_later.contains(&path_key(&file.relative_path)));
            claimed_later.extend(keys);
        }

        // The declared sizes are the source's word, so they only refuse early — before anything
        // is downloaded — a pack too large by its own account; staging counts the bytes.
        let mut declared: u64 = 0;
        for file in checked.iter().flat_map(|(_, files)| files) {
            declared = declared.saturating_add(file.declared_size);
            if declared > limits.max_override_bytes {
                return Err(InstanceError::FileTooLarge {
                    path: file.relative_path.clone(),
                    size: declared,
                    limit: limits.max_override_bytes,
                });
            }
        }

        Ok(PlanOverrides { sources: checked })
    }
}
