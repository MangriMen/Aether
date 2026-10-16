use crate::features::{
    instance::ProviderId,
    settings::{DefaultInstanceSettings, MemorySettings, Settings, SettingsError, WindowSettings},
};

/// How much each cache takes, and whether emptying them is refused now.
#[derive(Debug, PartialEq, Eq)]
pub struct CacheUsage {
    pub entries: Vec<CacheUsageEntry>,
    pub busy: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CacheUsageEntry {
    pub kind: crate::shared::housekeeping::CacheKind,
    pub bytes: u64,
}

/// What takes the disk, as the user tells it apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageKind {
    /// The game files the launcher keeps for all instances: assets, libraries, versions,
    /// natives.
    Minecraft,
    /// The Java runtimes the launcher downloaded, shared by all instances.
    Java,
    /// The instances' own files: mods, worlds, settings.
    Instances,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StorageUsageEntry {
    pub kind: StorageKind,
    pub bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StorageUsage {
    pub entries: Vec<StorageUsageEntry>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClearCacheOutcome {
    Cleared,
    /// Nothing was removed: the app is busy with something that uses the caches.
    Busy,
}

#[derive(Debug)]
pub struct EditSettings {
    pub max_concurrent_downloads: usize,
    pub platform_provider: Option<EditPlatformProvider>,
}

/// The choice of the provider serving `platform`; `None` drops the choice.
#[derive(Debug)]
pub struct EditPlatformProvider {
    pub platform: String,
    pub provider_id: Option<String>,
}

impl EditSettings {
    /// Refuse a provider choice that no platform install could ever use: an empty platform or a
    /// provider id that is not `plugin#capability`.
    pub fn validate(&self) -> Result<(), SettingsError> {
        let Some(choice) = &self.platform_provider else {
            return Ok(());
        };

        if choice.platform.trim().is_empty() {
            return Err(SettingsError::InvalidPlatformProvider {
                platform: choice.platform.clone(),
                reason: "the platform is empty".to_owned(),
            });
        }

        if let Some(provider_id) = &choice.provider_id {
            provider_id.parse::<ProviderId>().map_err(|reason| {
                SettingsError::InvalidPlatformProvider {
                    platform: choice.platform.clone(),
                    reason,
                }
            })?;
        }

        Ok(())
    }

    pub fn apply_to(self, settings: &mut Settings) -> bool {
        let mut is_changed = false;

        if settings.max_concurrent_downloads() != self.max_concurrent_downloads {
            settings.set_max_concurrent_downloads(self.max_concurrent_downloads);
            is_changed = true;
        }

        if let Some(choice) = self.platform_provider {
            is_changed |=
                settings.set_platform_provider(&choice.platform, choice.provider_id.as_deref());
        }

        is_changed
    }
}

#[derive(Debug)]
pub struct EditDefaultInstanceSettings {
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub window: Option<WindowSettings>,
    pub hooks: Option<EditHooks>,
}

#[derive(Debug)]
pub struct EditHooks {
    pub pre_launch: Option<String>,

    pub wrapper: Option<String>,

    pub post_exit: Option<String>,
}

impl EditDefaultInstanceSettings {
    pub fn apply_to(self, settings: &mut DefaultInstanceSettings) -> bool {
        let mut is_changed = false;

        if let Some(launch_args) = self.launch_args {
            is_changed |= settings.update_launch_args(launch_args);
        }

        if let Some(env_vars) = self.env_vars {
            is_changed |= settings.update_env_vars(env_vars);
        }

        if let Some(memory) = self.memory {
            is_changed |= settings.update_memory(memory);
        }

        if let Some(window) = self.window {
            is_changed |= settings.update_window(window);
        }

        if let Some(edit_hooks) = self.hooks {
            is_changed |= edit_hooks.apply_to(settings);
        }

        is_changed
    }
}

impl EditHooks {
    pub fn apply_to(self, settings: &mut DefaultInstanceSettings) -> bool {
        settings.update_hooks(self.pre_launch, self.wrapper, self.post_exit)
    }
}
