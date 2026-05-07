use crate::features::{
    instance::{Instance, InstanceError},
    settings::{Hooks, MemorySettings, WindowSize},
};

#[derive(Debug, sqlx::FromRow)]
pub struct SqlInstance {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub install_stage: String,

    // Minecraft Metadata
    pub game_version: String,
    pub loader: String,
    pub loader_version_json: Option<String>,

    // Launch Settings
    pub java_path: Option<String>,
    pub launch_args_json: Option<String>,
    pub env_vars_json: Option<String>,

    // MemorySettings (i64, так как в SQLite INTEGER)
    pub memory_maximum: i64,

    // WindowSize & Display
    pub force_fullscreen: bool,
    pub window_width: i64,
    pub window_height: i64,

    // Timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub last_played_at: Option<chrono::DateTime<chrono::Utc>>,

    // Stats
    pub time_played: i64,
    pub recent_time_played: i64,

    // Hooks
    pub hook_pre_launch: Option<String>,
    pub hook_wrapper: Option<String>,
    pub hook_post_exit: Option<String>,
}

impl From<Instance> for SqlInstance {
    fn from(i: Instance) -> Self {
        Self {
            id: i.id,
            name: i.name,
            icon_path: i.icon_path,
            install_stage: i.install_stage.as_str().to_string(),

            // Minecraft Metadata
            game_version: i.game_version,
            loader: serde_json::to_string(&i.loader).unwrap_or_default(),
            loader_version_json: i
                .loader_version
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap()),

            // Launch Settings
            java_path: i.java_path,
            launch_args_json: i
                .launch_args
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap()),
            env_vars_json: i
                .env_vars
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap()),

            // Runtime Settings (Flattened)
            memory_maximum: i.memory.map_or(2048, |m| i64::from(m.maximum)),
            force_fullscreen: i.force_fullscreen.unwrap_or(false),
            window_width: i.game_resolution.map_or(960, |r| i64::from(r.0)),
            window_height: i.game_resolution.map_or(540, |r| i64::from(r.1)),

            // Timestamps
            created_at: i.created,
            modified_at: i.modified,
            last_played_at: i.last_played,

            // Stats
            time_played: i.time_played.cast_signed(),
            recent_time_played: i.recent_time_played.cast_signed(),

            // Hooks (Flattened)
            hook_pre_launch: i.hooks.pre_launch().cloned(),
            hook_wrapper: i.hooks.wrapper().cloned(),
            hook_post_exit: i.hooks.post_exit().cloned(),
        }
    }
}

impl TryFrom<SqlInstance> for Instance {
    type Error = InstanceError;

    fn try_from(row: SqlInstance) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            icon_path: row.icon_path,
            install_stage: row
                .install_stage
                .parse()
                .map_err(|_| InstanceError::Storage("Invalid stage".into()))?,

            game_version: row.game_version,
            // Предполагаем, что ModLoader реализует Serialize/Deserialize
            loader: serde_json::from_str(&row.loader)
                .map_err(|e| InstanceError::Storage(e.to_string()))?,

            loader_version: row
                .loader_version_json
                .map(|s| serde_json::from_str(&s).unwrap()),

            java_path: row.java_path,
            launch_args: row
                .launch_args_json
                .map(|s| serde_json::from_str(&s).unwrap()),

            env_vars: row.env_vars_json.map(|s| serde_json::from_str(&s).unwrap()),

            memory: Some(MemorySettings::new(
                row.memory_maximum
                    .try_into()
                    .map_err(|_| InstanceError::Storage("Invalid memory".into()))?,
            )),
            force_fullscreen: Some(row.force_fullscreen),
            game_resolution: Some(WindowSize::new(
                row.window_width
                    .try_into()
                    .map_err(|_| InstanceError::Storage("Invalid width".into()))?,
                row.window_height
                    .try_into()
                    .map_err(|_| InstanceError::Storage("Invalid height".into()))?,
            )),

            created: row.created_at,
            modified: row.modified_at,
            last_played: row.last_played_at,

            time_played: row.time_played.cast_unsigned(),
            recent_time_played: row.recent_time_played.cast_unsigned(),

            hooks: Hooks::new(row.hook_pre_launch, row.hook_wrapper, row.hook_post_exit),

            // Это поле заполняется отдельно после запроса к instance_pack_info
            pack_info: None,
        })
    }
}
