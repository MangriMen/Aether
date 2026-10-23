use crate::features::instance::{
    Instance, InstanceChange, InstanceError, InstanceId, InstanceStorage,
};
use async_trait::async_trait;
use sqlx::{Sqlite, SqliteConnection, SqlitePool, Transaction};
use tracing::warn;

use super::{sql_instance::SqlInstance, sql_pack_info::SqlPackInfo};

pub struct SqliteInstanceStorage {
    pool: SqlitePool,
}

impl SqliteInstanceStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// A read-modify-write has to hold the write lock from the read on, or a concurrent write
    /// commits in between and is overwritten. `BEGIN IMMEDIATE` takes that lock up front.
    async fn begin_write(&self) -> Result<Transaction<'static, Sqlite>, InstanceError> {
        self.pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl InstanceStorage for SqliteInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                i.id, i.name, i.icon_path, i.install_stage, i.game_version, i.loader, 
                i.loader_version_json, 
                i.override_java_path, i.java_path,
                i.override_launch_args, i.launch_args_json,
                i.override_env_vars, i.env_vars_json,
                i.override_memory, i.memory_maximum, 
                i.override_window_settings, i.force_fullscreen, i.window_width, i.window_height, 
                i.created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                i.modified_at as "modified_at: chrono::DateTime<chrono::Utc>", 
                i.last_played_at as "last_played_at?: chrono::DateTime<chrono::Utc>", 
                i.time_played, i.recent_time_played, 
                i.override_hooks, i.hook_pre_launch, i.hook_wrapper, i.hook_post_exit,
                p.provider_id as "pack_provider_id?", 
                p.modpack_id as "pack_modpack_id?", 
                p.version_id as "pack_version_id?"
            FROM instances i
            LEFT JOIN instance_pack_info p ON i.id = p.instance_id
            ORDER BY i.modified_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let sql_instance = SqlInstance {
                    id: row.id,

                    name: row.name,

                    icon_path: row.icon_path,

                    install_stage: row.install_stage,

                    game_version: row.game_version,
                    loader: row.loader,
                    loader_version_json: row.loader_version_json,

                    override_java_path: row.override_java_path,
                    java_path: row.java_path,

                    override_launch_args: row.override_launch_args,
                    launch_args_json: row.launch_args_json,

                    override_env_vars: row.override_env_vars,
                    env_vars_json: row.env_vars_json,

                    override_memory: row.override_memory,
                    memory_maximum: row.memory_maximum,

                    override_window_settings: row.override_window_settings,
                    force_fullscreen: row.force_fullscreen,
                    window_width: row.window_width,
                    window_height: row.window_height,

                    created_at: row.created_at,
                    modified_at: row.modified_at,
                    last_played_at: row.last_played_at,

                    time_played: row.time_played,
                    recent_time_played: row.recent_time_played,

                    override_hooks: row.override_hooks,
                    hook_pre_launch: row.hook_pre_launch,
                    hook_wrapper: row.hook_wrapper,
                    hook_post_exit: row.hook_post_exit,
                };

                let row_id = sql_instance.id.clone();
                let mut instance = match Instance::try_from(sql_instance) {
                    Ok(instance) => instance,
                    Err(err) => {
                        warn!("Failed to read instance {:?}: {:?}", row_id, err);
                        return None;
                    }
                };

                if let (Some(pid), Some(mid), Some(vid)) = (
                    row.pack_provider_id,
                    row.pack_modpack_id,
                    row.pack_version_id,
                ) {
                    instance.pack_info = Some(
                        SqlPackInfo {
                            provider_id: pid,
                            modpack_id: mid,
                            version_id: vid,
                        }
                        .into(),
                    );
                }

                Some(instance)
            })
            .collect())
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        read_instance(&mut conn, id).await
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut tx = self.begin_write().await?;
        write_instance(&mut tx, instance).await?;
        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }

    async fn update(
        &self,
        id: &InstanceId,
        change: InstanceChange<'_>,
    ) -> Result<Instance, InstanceError> {
        let mut tx = self.begin_write().await?;
        let mut instance = read_instance(&mut tx, id).await?;
        if change(&mut instance)? {
            write_instance(&mut tx, &instance).await?;
        }
        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        Ok(instance)
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let mut tx = self.begin_write().await?;
        sqlx::query!("DELETE FROM instances WHERE id = ?", id.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }
}

#[allow(clippy::too_many_lines)]
async fn read_instance(
    conn: &mut SqliteConnection,
    id: &InstanceId,
) -> Result<Instance, InstanceError> {
    let row = sqlx::query!(
        r#"
            SELECT 
                i.id, i.name, i.icon_path, i.install_stage, i.game_version, i.loader, 
                i.loader_version_json, 
                i.override_java_path, i.java_path,
                i.override_launch_args, i.launch_args_json,
                i.override_env_vars, i.env_vars_json, 
                i.override_memory, i.memory_maximum, 
                i.override_window_settings, i.force_fullscreen, i.window_width, i.window_height, 
                i.created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                i.modified_at as "modified_at: chrono::DateTime<chrono::Utc>", 
                i.last_played_at as "last_played_at?: chrono::DateTime<chrono::Utc>", 
                i.time_played, i.recent_time_played, 
                i.override_hooks, i.hook_pre_launch, i.hook_wrapper, i.hook_post_exit,
                p.provider_id as "pack_provider_id?", 
                p.modpack_id as "pack_modpack_id?", 
                p.version_id as "pack_version_id?"
            FROM instances i
            LEFT JOIN instance_pack_info p ON i.id = p.instance_id
            WHERE i.id = ?
            "#,
        id.as_str()
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| InstanceError::Storage(e.to_string()))?;

    let row = row.ok_or_else(|| InstanceError::NotFound {
        instance_id: id.to_string(),
    })?;

    let sql_instance = SqlInstance {
        id: row.id,

        name: row.name,

        icon_path: row.icon_path,

        install_stage: row.install_stage,

        game_version: row.game_version,
        loader: row.loader,
        loader_version_json: row.loader_version_json,

        override_java_path: row.override_java_path,
        java_path: row.java_path,

        override_launch_args: row.override_launch_args,
        launch_args_json: row.launch_args_json,

        override_env_vars: row.override_env_vars,
        env_vars_json: row.env_vars_json,

        override_memory: row.override_memory,
        memory_maximum: row.memory_maximum,

        override_window_settings: row.override_window_settings,
        force_fullscreen: row.force_fullscreen,
        window_width: row.window_width,
        window_height: row.window_height,

        created_at: row.created_at,
        modified_at: row.modified_at,
        last_played_at: row.last_played_at,

        time_played: row.time_played,
        recent_time_played: row.recent_time_played,

        override_hooks: row.override_hooks,
        hook_pre_launch: row.hook_pre_launch,
        hook_wrapper: row.hook_wrapper,
        hook_post_exit: row.hook_post_exit,
    };

    let mut instance = Instance::try_from(sql_instance)?;

    if let (Some(pid), Some(mid), Some(vid)) = (
        row.pack_provider_id,
        row.pack_modpack_id,
        row.pack_version_id,
    ) {
        instance.pack_info = Some(
            SqlPackInfo {
                provider_id: pid,
                modpack_id: mid,
                version_id: vid,
            }
            .into(),
        );
    }

    Ok(instance)
}

#[allow(clippy::too_many_lines)]
async fn write_instance(
    tx: &mut SqliteConnection,
    instance: &Instance,
) -> Result<(), InstanceError> {
    let sql = SqlInstance::from(instance.clone());

    sqlx::query!(
            r#"
            INSERT INTO instances (
                id, name, icon_path, install_stage, game_version, loader, 
                loader_version_json, 
                override_java_path, java_path, 
                override_launch_args, launch_args_json, 
                override_env_vars, env_vars_json, 
                override_memory, memory_maximum, 
                override_window_settings, force_fullscreen, window_width, window_height, 
                created_at, modified_at, last_played_at, time_played, 
                recent_time_played, override_hooks, hook_pre_launch, hook_wrapper, hook_post_exit
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name, 
                
                icon_path = excluded.icon_path, 
                
                install_stage = excluded.install_stage,
                
                game_version = excluded.game_version, 
                loader = excluded.loader,
                loader_version_json = excluded.loader_version_json, 
                
                override_java_path = excluded.override_java_path,
                java_path = excluded.java_path,
                
                override_launch_args = excluded.override_launch_args,
                launch_args_json = excluded.launch_args_json, 
                
                override_env_vars = excluded.override_env_vars,
                env_vars_json = excluded.env_vars_json,
                
                override_memory = excluded.override_memory,
                memory_maximum = excluded.memory_maximum, 
                
                override_window_settings = excluded.override_window_settings,
                force_fullscreen = excluded.force_fullscreen,
                window_width = excluded.window_width, 
                window_height = excluded.window_height,
                
                modified_at = excluded.modified_at, 
                last_played_at = excluded.last_played_at,
                
                time_played = excluded.time_played, 
                recent_time_played = excluded.recent_time_played,
                
                override_hooks = excluded.override_hooks,
                hook_pre_launch = excluded.hook_pre_launch, 
                hook_wrapper = excluded.hook_wrapper,
                hook_post_exit = excluded.hook_post_exit
            "#,
            sql.id,
            sql.name,
            sql.icon_path,
            sql.install_stage,
            sql.game_version,
            sql.loader,
            sql.loader_version_json,
            sql.override_java_path,
            sql.java_path,
            sql.override_launch_args,
            sql.launch_args_json,
            sql.override_env_vars,
            sql.env_vars_json,
            sql.override_memory,
            sql.memory_maximum,
            sql.override_window_settings,
            sql.force_fullscreen,
            sql.window_width,
            sql.window_height,
            sql.created_at,
            sql.modified_at,
            sql.last_played_at,
            sql.time_played,
            sql.recent_time_played,
            sql.override_hooks,
            sql.hook_pre_launch,
            sql.hook_wrapper,
            sql.hook_post_exit
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

    let instance_id = instance.id().as_str();

    match &instance.pack_info {
            Some(pack) => {
                let sql_pack = SqlPackInfo::from(pack.clone());
                sqlx::query!(
                    r#"
                    INSERT INTO instance_pack_info (instance_id, provider_id, modpack_id, version_id)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(instance_id) DO UPDATE SET
                        provider_id = excluded.provider_id,
                        modpack_id = excluded.modpack_id,
                        version_id = excluded.version_id
                    "#,
                    instance_id, sql_pack.provider_id, sql_pack.modpack_id, sql_pack.version_id
                )
                .execute(&mut *tx)
                .await
            }
            None => {
                sqlx::query!("DELETE FROM instance_pack_info WHERE instance_id = ?", instance_id)
                .execute(&mut *tx)
                .await
            }
        }
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

    Ok(())
}
