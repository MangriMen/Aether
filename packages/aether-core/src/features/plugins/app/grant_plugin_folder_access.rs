use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::features::{
    plugins::{
        AllowedPath, AllowedPathRejection, PluginError, PluginRegistry, PluginSettingsStorage,
        PluginState, core_folders, is_filesystem_root, partition_mountable,
    },
    settings::LocationInfo,
};

use super::{
    plugin_access_grant::revoke_in_settings,
    ports::{GrantPluginFolderAccessUseCasePort, ReloadPluginUseCasePort},
};

/// What became of the folder of a file the user named for a plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderAccess {
    /// The plugin could already read the file.
    AlreadyAllowed { folder: PathBuf },
    /// A dry run: the plugin cannot read the file yet, and nothing was changed.
    Needed { folder: PathBuf },
    /// The folder was added read-only, and the plugin restarted with it if it was running.
    Granted { folder: PathBuf },
}

/// Gives a plugin read-only access to the folder of a file the user named for it (the pack an
/// importer is to read). The whole folder, because a pack is more than its `pack.toml`; a dry
/// run tells the UI whether there is anything to ask, and refuses what the grant would refuse.
///
/// Folders are mounted when an instance is built, so a running plugin is restarted. A busy
/// plugin gets neither the folder nor the restart: the caller gets [`PluginError::Busy`]. Nor
/// does a plugin being loaded, which may or may not read the folder: the caller gets
/// [`PluginError::LoadingInProgress`].
pub(crate) struct GrantPluginFolderAccessUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    reload_plugin: Arc<dyn ReloadPluginUseCasePort>,
    location_info: Arc<LocationInfo>,
}

impl GrantPluginFolderAccessUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        reload_plugin: Arc<dyn ReloadPluginUseCasePort>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
            reload_plugin,
            location_info,
        }
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: String,
        file: PathBuf,
        dry_run: bool,
    ) -> Result<FolderAccess, PluginError> {
        let folder = folder_of(&plugin_id, &file)?;
        let added = AllowedPath::read_only_folder(&folder);

        let core_folders = core_folders(&self.location_info, &plugin_id);
        added.validate_against(&core_folders).map_err(|reason| {
            PluginError::InvalidAllowedPath {
                plugin_id: plugin_id.clone(),
                path: added.host.clone(),
                reason,
            }
        })?;

        let saved = {
            let _settings_lock = self.plugin_registry.lock_settings(&plugin_id).await;
            let mut settings = self
                .plugin_settings_storage
                .get(&plugin_id)
                .await?
                .unwrap_or_default();
            // Only a folder the loader mounts counts: one it leaves out (a drive, a folder of the
            // core's) is in the settings and the plugin still cannot read the file.
            let in_settings = partition_mountable(&settings.allowed_paths, &core_folders)
                .0
                .into_iter()
                .any(|path| path.exposes(&file));

            // What counts is what the plugin can open now: a folder saved while it ran is not
            // mounted until it restarts.
            if in_settings
                && self
                    .mounts(&plugin_id, &file)?
                    .is_none_or(|mounted| mounted)
            {
                return Ok(FolderAccess::AlreadyAllowed { folder });
            }
            if dry_run {
                return Ok(FolderAccess::Needed { folder });
            }

            if !in_settings {
                settings.allowed_paths.push(added.clone());
                self.plugin_settings_storage
                    .upsert(&plugin_id, &settings)
                    .await?;
            }
            !in_settings
        };

        // Read again after the save: a load started in between may have read the settings
        // either before or after it.
        let result = match self.mounts(&plugin_id, &file) {
            Ok(Some(true) | None) => return Ok(FolderAccess::Granted { folder }),
            Ok(Some(false)) => self.reload_plugin.execute(plugin_id.clone()).await,
            Err(err) => Err(err),
        };

        if let Err(err) = result {
            // The folder goes only with the restart that brings it in: left in the settings,
            // it would turn up unasked at the next load, after the user was told it failed.
            if saved {
                revoke_in_settings(
                    &self.plugin_registry,
                    self.plugin_settings_storage.as_ref(),
                    &plugin_id,
                    |settings| {
                        if let Some(at) = settings.allowed_paths.iter().rposition(|p| *p == added) {
                            settings.allowed_paths.remove(at);
                        }
                    },
                )
                .await?;
            }
            return Err(err);
        }

        Ok(FolderAccess::Granted { folder })
    }

    /// Whether the running instance has `file` mounted; `None` when no instance runs, and the
    /// settings are what its next load gets. A load under way may or may not have read them yet,
    /// so it is refused rather than guessed at.
    fn mounts(&self, plugin_id: &str, file: &Path) -> Result<Option<bool>, PluginError> {
        match &self.plugin_registry.get(plugin_id)?.state {
            PluginState::Loaded(handle) => Ok(Some(handle.folders().exposes(file))),
            PluginState::Loading => Err(PluginError::LoadingInProgress {
                plugin_id: plugin_id.to_owned(),
            }),
            PluginState::NotLoaded
            | PluginState::Unloading
            | PluginState::Removing
            | PluginState::Failed(_)
            | PluginState::Incompatible(_) => Ok(None),
        }
    }
}

/// The folder holding `file`, as the user named it: the importer hands the plugin the file under
/// that same name, so a mount of a resolved (symlink-free) path would not answer for it.
fn folder_of(plugin_id: &str, file: &Path) -> Result<PathBuf, PluginError> {
    let rejected = |reason| PluginError::InvalidAllowedPath {
        plugin_id: plugin_id.to_owned(),
        path: file.to_string_lossy().into_owned(),
        reason,
    };

    if !file.is_absolute() {
        return Err(rejected(AllowedPathRejection::HostNotAbsolute));
    }

    let folder = file
        .parent()
        .filter(|folder| folder.is_dir())
        .ok_or_else(|| rejected(AllowedPathRejection::HostNotAFolder))?;

    // A file at the top of a drive would take the whole drive with it, however the path leads
    // there (`D:\x\..\pack.toml` is at the top of `D:`).
    if is_filesystem_root(folder) {
        return Err(rejected(AllowedPathRejection::HostIsRoot));
    }

    Ok(folder.to_path_buf())
}

#[async_trait]
impl GrantPluginFolderAccessUseCasePort for GrantPluginFolderAccessUseCase {
    async fn execute(
        &self,
        plugin_id: String,
        file: PathBuf,
        dry_run: bool,
    ) -> Result<FolderAccess, PluginError> {
        self.execute(plugin_id, file, dry_run).await
    }
}
