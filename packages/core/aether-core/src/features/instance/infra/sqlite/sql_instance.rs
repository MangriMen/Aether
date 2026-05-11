use chrono::{DateTime, Utc};

use crate::{
    features::{
        instance::{Instance, InstanceError, InstanceSnapshot},
        settings::{Hooks, MemorySettings, WindowSettings, WindowSize},
    },
    shared::Overridable,
};

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
            install_stage: s.install_stage.as_str().to_string(),

            game_version: s.game_version,
            loader: serde_json::to_string(&s.loader).unwrap_or_default(),
            loader_version_json: s.loader_version.map(|v| serde_json::to_string(&v).unwrap()),

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
            install_stage: row
                .install_stage
                .parse()
                .map_err(|_| InstanceError::Storage("Invalid stage".into()))?,

            game_version: row.game_version,
            loader: serde_json::from_str(&row.loader)
                .map_err(|e| InstanceError::Storage(e.to_string()))?,
            loader_version: row
                .loader_version_json
                .map(|s| serde_json::from_str(&s).unwrap()),

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
