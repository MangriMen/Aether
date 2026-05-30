use chrono::{DateTime, Utc};

use crate::{
    features::{
        instance::{
            InstanceError, InstanceSnapshot,
            domain::{InstanceField, InstanceValidationErrorReason},
        },
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::{Hooks, MemorySettings, WindowSettings},
    },
    shared::overridable::domain::Overridable,
};

use super::{InstanceInstallStage, PackInfo};

#[derive(Debug, Clone)]
pub struct Instance {
    id: String,

    name: String,
    icon_path: Option<String>,

    pub install_stage: InstanceInstallStage,

    // Metadata
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,

    // Launch arguments
    pub java_path: Overridable<String>,
    pub launch_args: Overridable<Vec<String>>,
    pub env_vars: Overridable<Vec<(String, String)>>,

    // Runtime settings
    pub memory: Overridable<MemorySettings>,

    // Window settings
    pub window: Overridable<WindowSettings>,

    // Timestamps
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    // Stats
    pub time_played: u64,
    pub recent_time_played: u64,

    // Hooks
    pub hooks: Overridable<Hooks>,

    pub pack_info: Option<PackInfo>,
}

impl Instance {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn icon_path(&self) -> Option<&str> {
        self.icon_path.as_deref()
    }

    pub fn rename(&mut self, new_name: String) -> Result<bool, InstanceError> {
        let trimmed = new_name.trim();

        if trimmed.is_empty() {
            return Err(InstanceError::ValidationError {
                field: InstanceField::Name,
                reason: InstanceValidationErrorReason::CannotBeEmpty,
            });
        }

        if self.name == trimmed {
            return Ok(false);
        }

        self.name = new_name;
        self.mark_modified();

        Ok(true)
    }

    pub fn set_icon(&mut self, icon_id: Option<String>) {
        self.icon_path = icon_id;
        self.mark_modified();
    }

    fn mark_modified(&mut self) {
        self.modified = Utc::now();
    }

    pub fn update_java_path_active(&mut self, is_active: bool) -> bool {
        if self.java_path.is_active != is_active {
            self.java_path.is_active = is_active;
            return true;
        }
        false
    }

    pub fn update_java_path_data(&mut self, data: String) -> bool {
        if self.java_path.data != data {
            self.java_path.data = data;
            return true;
        }
        false
    }

    pub fn update_launch_args_active(&mut self, is_active: bool) -> bool {
        if self.launch_args.is_active != is_active {
            self.launch_args.is_active = is_active;
            return true;
        }
        false
    }

    pub fn update_launch_args_data(&mut self, data: Vec<String>) -> bool {
        if self.launch_args.data != data {
            self.launch_args.data = data;
            return true;
        }
        false
    }

    pub fn update_env_vars_active(&mut self, is_active: bool) -> bool {
        if self.env_vars.is_active != is_active {
            self.env_vars.is_active = is_active;
            return true;
        }
        false
    }

    pub fn update_env_vars_data(&mut self, data: Vec<(String, String)>) -> bool {
        if self.env_vars.data != data {
            self.env_vars.data = data;
            return true;
        }
        false
    }

    pub fn update_memory_active(&mut self, is_active: bool) -> bool {
        if self.memory.is_active != is_active {
            self.memory.is_active = is_active;
            return true;
        }
        false
    }

    pub fn update_memory_data(&mut self, data: MemorySettings) -> bool {
        if self.memory.data != data {
            self.memory.data = data;
            return true;
        }
        false
    }

    pub fn update_window_active(&mut self, is_active: bool) -> bool {
        if self.window.is_active != is_active {
            self.window.is_active = is_active;
            return true;
        }
        false
    }

    pub fn update_window_data(&mut self, data: WindowSettings) -> bool {
        if self.window.data != data {
            self.window.data = data;
            return true;
        }
        false
    }

    pub fn update_hooks_active(&mut self, is_active: bool) -> bool {
        if self.hooks.is_active != is_active {
            self.hooks.is_active = is_active;
            return true;
        }
        false
    }

    // Помнишь, мы перенесли update_hooks внутрь самих Hooks?
    // Теперь это просто проброс чистого метода
    pub fn update_hooks_data(
        &mut self,
        pre_launch: Option<String>,
        wrapper: Option<String>,
        post_exit: Option<String>,
    ) -> bool {
        self.hooks.data.update_hooks(pre_launch, wrapper, post_exit)
    }

    pub fn from_snapshot(snapshot: InstanceSnapshot) -> Self {
        Self {
            id: snapshot.id,

            name: snapshot.name,

            icon_path: snapshot.icon_path,

            install_stage: snapshot.install_stage,

            game_version: snapshot.game_version,
            loader: snapshot.loader,
            loader_version: snapshot.loader_version,

            java_path: snapshot.java_path,
            launch_args: snapshot.launch_args,
            env_vars: snapshot.env_vars,

            memory: snapshot.memory,

            window: snapshot.window,

            created: snapshot.created,
            modified: snapshot.modified,
            last_played: snapshot.last_played,

            time_played: snapshot.time_played,
            recent_time_played: snapshot.recent_time_played,

            hooks: snapshot.hooks,

            pack_info: snapshot.pack_info,
        }
    }

    pub fn snapshot(&self) -> InstanceSnapshot {
        InstanceSnapshot {
            id: self.id.clone(),

            name: self.name.clone(),

            icon_path: self.icon_path.clone(),

            install_stage: self.install_stage,

            game_version: self.game_version.clone(),
            loader: self.loader,
            loader_version: self.loader_version.clone(),

            java_path: self.java_path.clone(),
            launch_args: self.launch_args.clone(),
            env_vars: self.env_vars.clone(),

            memory: self.memory.clone(),

            window: self.window.clone(),

            created: self.created,
            modified: self.modified,
            last_played: self.last_played,

            time_played: self.time_played,
            recent_time_played: self.recent_time_played,

            hooks: self.hooks.clone(),

            pack_info: self.pack_info.clone(),
        }
    }
}

pub struct InstanceBuilder {
    id: String,
    name: String,

    game_version: String,
    loader: ModLoader,
    loader_version: Option<LoaderVersionPreference>,

    icon_path: Option<String>,

    pack_info: Option<PackInfo>,
}

impl InstanceBuilder {
    pub fn new(id: String, name: String, game_version: String, loader: ModLoader) -> Self {
        Self {
            name,
            id,
            game_version,
            loader,
            icon_path: None,
            loader_version: None,
            pack_info: None,
        }
    }

    #[must_use]
    pub fn with_loader_version(mut self, loader_version: Option<LoaderVersionPreference>) -> Self {
        self.loader_version = loader_version;
        self
    }

    #[must_use]
    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon_path = icon;
        self
    }

    #[must_use]
    pub fn with_pack_info(mut self, info: Option<PackInfo>) -> Self {
        self.pack_info = info;
        self
    }

    #[must_use]
    pub fn build(self) -> Instance {
        let now = Utc::now();
        Instance {
            id: self.id,

            name: self.name,

            icon_path: self.icon_path,

            game_version: self.game_version,
            loader: self.loader,
            loader_version: self.loader_version,

            install_stage: InstanceInstallStage::NotInstalled,

            created: now,
            modified: now,

            // All other fields are default
            java_path: Overridable::new_disabled_default(),
            launch_args: Overridable::new_disabled_default(),
            env_vars: Overridable::new_disabled_default(),

            memory: Overridable::new_disabled_default(),

            window: Overridable::new_disabled_default(),

            last_played: None,

            time_played: 0,
            recent_time_played: 0,

            hooks: Overridable::new_disabled_default(),

            pack_info: self.pack_info,
        }
    }
}
