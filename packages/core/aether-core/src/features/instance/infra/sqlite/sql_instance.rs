use chrono::{DateTime, Utc};

use crate::{
    features::{
        instance::{Instance, InstanceError, InstanceInstallStage, InstanceSnapshot},
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::{Hooks, MemorySettings, WindowSettings, WindowSize},
    },
    shared::overridable::domain::Overridable,
};

/// DTO for `ModLoader` serialization in `SQLite` storage.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModLoaderDto {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl From<ModLoaderDto> for ModLoader {
    fn from(value: ModLoaderDto) -> Self {
        match value {
            ModLoaderDto::Vanilla => Self::Vanilla,
            ModLoaderDto::Forge => Self::Forge,
            ModLoaderDto::Fabric => Self::Fabric,
            ModLoaderDto::Quilt => Self::Quilt,
            ModLoaderDto::NeoForge => Self::NeoForge,
        }
    }
}

impl From<ModLoader> for ModLoaderDto {
    fn from(value: ModLoader) -> Self {
        match value {
            ModLoader::Vanilla => Self::Vanilla,
            ModLoader::Forge => Self::Forge,
            ModLoader::Fabric => Self::Fabric,
            ModLoader::Quilt => Self::Quilt,
            ModLoader::NeoForge => Self::NeoForge,
        }
    }
}

/// DTO for `InstanceInstallStage` serialization in `SQLite` storage.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceInstallStageDto {
    Installed,
    Installing,
    PackInstalling,
    NotInstalled,
}

impl From<InstanceInstallStageDto> for InstanceInstallStage {
    fn from(value: InstanceInstallStageDto) -> Self {
        match value {
            InstanceInstallStageDto::Installed => Self::Installed,
            InstanceInstallStageDto::Installing => Self::Installing,
            InstanceInstallStageDto::PackInstalling => Self::PackInstalling,
            InstanceInstallStageDto::NotInstalled => Self::NotInstalled,
        }
    }
}

impl From<InstanceInstallStage> for InstanceInstallStageDto {
    fn from(value: InstanceInstallStage) -> Self {
        match value {
            InstanceInstallStage::Installed => Self::Installed,
            InstanceInstallStage::Installing => Self::Installing,
            InstanceInstallStage::PackInstalling => Self::PackInstalling,
            InstanceInstallStage::NotInstalled => Self::NotInstalled,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreferenceDto {
    Latest,
    Stable,
    #[serde(untagged)]
    Exact(String),
}

impl From<LoaderVersionPreferenceDto> for LoaderVersionPreference {
    fn from(value: LoaderVersionPreferenceDto) -> Self {
        match value {
            LoaderVersionPreferenceDto::Latest => Self::Latest,
            LoaderVersionPreferenceDto::Stable => Self::Stable,
            LoaderVersionPreferenceDto::Exact(x) => Self::Exact(x),
        }
    }
}

impl From<LoaderVersionPreference> for LoaderVersionPreferenceDto {
    fn from(value: LoaderVersionPreference) -> Self {
        match value {
            LoaderVersionPreference::Latest => Self::Latest,
            LoaderVersionPreference::Stable => Self::Stable,
            LoaderVersionPreference::Exact(x) => Self::Exact(x),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
#[allow(clippy::struct_excessive_bools)]
pub struct SqlInstance {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub install_stage: String,

    // Minecraft Metadata
    pub game_version: String,
    pub loader: String,
    pub loader_version_json: Option<String>,

    // Launch arguments
    pub override_java_path: bool,
    pub java_path: String,

    pub override_launch_args: bool,
    pub launch_args_json: String,

    pub override_env_vars: bool,
    pub env_vars_json: String,

    // Runtime settings
    pub override_memory: bool,
    pub memory_maximum: i64,

    // WindowSize & Display
    pub override_window_settings: bool,
    pub force_fullscreen: bool,
    pub window_width: i64,
    pub window_height: i64,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub last_played_at: Option<DateTime<Utc>>,

    // Stats
    pub time_played: i64,
    pub recent_time_played: i64,

    // Hooks (Flattened)
    pub override_hooks: bool,
    pub hook_pre_launch: String,
    pub hook_wrapper: String,
    pub hook_post_exit: String,
}

impl From<Instance> for SqlInstance {
    fn from(i: Instance) -> Self {
        let s = i.snapshot();

        Self {
            id: s.id,
            name: s.name,
            icon_path: s.icon_path,
            install_stage: {
                let dto: InstanceInstallStageDto = s.install_stage.into();
                serde_json::to_string(&dto)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned()
            },

            game_version: s.game_version,
            loader: {
                let dto: ModLoaderDto = s.loader.into();
                serde_json::to_string(&dto)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned()
            },
            loader_version_json: s.loader_version.map(|v| {
                let dto: LoaderVersionPreferenceDto = v.into();
                serde_json::to_string(&dto).unwrap()
            }),

            // Java
            override_java_path: s.java_path.is_active,
            java_path: s.java_path.data,

            // Args & Env
            override_launch_args: s.launch_args.is_active,
            launch_args_json: serde_json::to_string(&s.launch_args.data)
                .unwrap_or_else(|_| "[]".into()),
            override_env_vars: s.env_vars.is_active,
            env_vars_json: serde_json::to_string(&s.env_vars.data).unwrap_or_else(|_| "[]".into()),

            // Memory
            override_memory: s.memory.is_active,
            memory_maximum: i64::from(s.memory.data.maximum),

            // Window
            override_window_settings: s.window.is_active,
            force_fullscreen: s.window.data.force_fullscreen(),
            window_width: i64::from(s.window.data.game_resolution().width()),
            window_height: i64::from(s.window.data.game_resolution().height()),

            created_at: s.created,
            modified_at: s.modified,
            last_played_at: s.last_played,

            time_played: s.time_played.cast_signed(),
            recent_time_played: s.recent_time_played.cast_signed(),

            // Hooks
            override_hooks: s.hooks.is_active,
            hook_pre_launch: s.hooks.data.pre_launch().to_string(),
            hook_wrapper: s.hooks.data.wrapper().to_string(),
            hook_post_exit: s.hooks.data.post_exit().to_string(),
        }
    }
}

impl TryFrom<SqlInstance> for Instance {
    type Error = InstanceError;

    fn try_from(row: SqlInstance) -> Result<Self, Self::Error> {
        let max_memory = u32::try_from(row.memory_maximum);
        let memory = if let Ok(max_memory) = max_memory {
            MemorySettings::new(max_memory)
        } else {
            MemorySettings::default()
        };

        let width = u16::try_from(row.window_width);
        let height = u16::try_from(row.window_height);

        let window_size = if let Ok(width) = width
            && let Ok(height) = height
        {
            WindowSize::new(width, height)
        } else {
            WindowSize::default()
        };

        let snapshot = InstanceSnapshot {
            id: row.id,
            name: row.name,
            icon_path: row.icon_path,
            install_stage: {
                let dto: InstanceInstallStageDto =
                    serde_json::from_str(&format!("\"{}\"", row.install_stage.trim_matches('"')))
                        .map_err(|_| InstanceError::Storage("Invalid stage".into()))?;
                dto.into()
            },

            game_version: row.game_version,
            loader: {
                let dto: ModLoaderDto =
                    serde_json::from_str(&format!("\"{}\"", row.loader.trim_matches('"')))
                        .map_err(|_| InstanceError::Storage("Invalid mod loader".into()))?;
                dto.into()
            },
            loader_version: row.loader_version_json.map(|s| {
                let dto: LoaderVersionPreferenceDto = serde_json::from_str(&s).unwrap();
                dto.into()
            }),

            // Java
            java_path: Overridable::new(row.java_path, row.override_java_path),

            // Launch Args
            launch_args: Overridable::new(
                serde_json::from_str(&row.launch_args_json).unwrap_or_default(),
                row.override_launch_args,
            ),

            // Env Vars
            env_vars: Overridable::new(
                serde_json::from_str(&row.env_vars_json).unwrap_or_default(),
                row.override_env_vars,
            ),

            // Memory
            memory: Overridable::new(memory, row.override_memory),

            // Window
            window: Overridable::new(
                WindowSettings::new(row.force_fullscreen, window_size),
                row.override_window_settings,
            ),

            // Hooks
            hooks: Overridable::new(
                Hooks::new(row.hook_pre_launch, row.hook_wrapper, row.hook_post_exit),
                row.override_hooks,
            ),

            created: row.created_at,
            modified: row.modified_at,
            last_played: row.last_played_at,
            time_played: row.time_played.cast_unsigned(),
            recent_time_played: row.recent_time_played.cast_unsigned(),

            // Load separately
            pack_info: None,
        };

        Ok(Instance::from_snapshot(snapshot))
    }
}
