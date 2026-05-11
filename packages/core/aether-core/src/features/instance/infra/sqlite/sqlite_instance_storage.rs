use crate::features::instance::{Instance, InstanceError, InstanceStorage};
use async_trait::async_trait;
use sqlx::SqlitePool;

use super::{sql_instance::SqlInstance, sql_pack_info::SqlPackInfo};

pub struct SqliteInstanceStorage {
    pool: SqlitePool,
}

impl SqliteInstanceStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InstanceStorage for SqliteInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                i.id, i.name, i.icon_path, i.install_stage, i.game_version, i.loader, 
                i.loader_version_json, i.java_path, i.launch_args_json, i.env_vars_json, 
                i.memory_maximum, i.force_fullscreen, i.window_width, i.window_height, 
                i.created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                i.modified_at as "modified_at: chrono::DateTime<chrono::Utc>", 
                i.last_played_at as "last_played_at?: chrono::DateTime<chrono::Utc>", 
                i.time_played, i.recent_time_played, 
                i.hook_pre_launch, i.hook_wrapper, i.hook_post_exit,
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

        rows.into_iter()
            .map(|row| {
                let sql_instance = SqlInstance {
                    id: row.id,
                    name: row.name,
                    icon_path: row.icon_path,
                    install_stage: row.install_stage,
                    game_version: row.game_version,
                    loader: row.loader,
                    loader_version_json: row.loader_version_json,
                    java_path: row.java_path,
                    launch_args_json: row.launch_args_json,
                    env_vars_json: row.env_vars_json,
                    memory_maximum: row.memory_maximum,
                    force_fullscreen: row.force_fullscreen,
                    window_width: row.window_width,
                    window_height: row.window_height,
                    created_at: row.created_at,
                    modified_at: row.modified_at,
                    last_played_at: row.last_played_at,
                    time_played: row.time_played,
                    recent_time_played: row.recent_time_played,
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
            })
            .collect()
    }

    async fn get(&self, id: &str) -> Result<Instance, InstanceError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                i.id, i.name, i.icon_path, i.install_stage, i.game_version, i.loader, 
                i.loader_version_json, i.java_path, i.launch_args_json, i.env_vars_json, 
                i.memory_maximum, i.force_fullscreen, i.window_width, i.window_height, 
                i.created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                i.modified_at as "modified_at: chrono::DateTime<chrono::Utc>", 
                i.last_played_at as "last_played_at?: chrono::DateTime<chrono::Utc>", 
                i.time_played, i.recent_time_played, 
                i.hook_pre_launch, i.hook_wrapper, i.hook_post_exit,
                p.provider_id as "pack_provider_id?", 
                p.modpack_id as "pack_modpack_id?", 
                p.version_id as "pack_version_id?"
            FROM instances i
            LEFT JOIN instance_pack_info p ON i.id = p.instance_id
            WHERE i.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
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
            java_path: row.java_path,
            launch_args_json: row.launch_args_json,
            env_vars_json: row.env_vars_json,
            memory_maximum: row.memory_maximum,
            force_fullscreen: row.force_fullscreen,
            window_width: row.window_width,
            window_height: row.window_height,
            created_at: row.created_at,
            modified_at: row.modified_at,
            last_played_at: row.last_played_at,
            time_played: row.time_played,
            recent_time_played: row.recent_time_played,
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

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let sql = SqlInstance::from(instance.clone());

        sqlx::query!(
            r#"
            INSERT INTO instances (
                id, name, icon_path, install_stage, game_version, loader, 
                loader_version_json, java_path, launch_args_json, env_vars_json, 
                memory_maximum, force_fullscreen, window_width, window_height, 
                created_at, modified_at, last_played_at, time_played, 
                recent_time_played, hook_pre_launch, hook_wrapper, hook_post_exit
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name, icon_path = excluded.icon_path, install_stage = excluded.install_stage,
                game_version = excluded.game_version, loader = excluded.loader,
                loader_version_json = excluded.loader_version_json, java_path = excluded.java_path,
                launch_args_json = excluded.launch_args_json, env_vars_json = excluded.env_vars_json,
                memory_maximum = excluded.memory_maximum, force_fullscreen = excluded.force_fullscreen,
                window_width = excluded.window_width, window_height = excluded.window_height,
                modified_at = excluded.modified_at, last_played_at = excluded.last_played_at,
                time_played = excluded.time_played, recent_time_played = excluded.recent_time_played,
                hook_pre_launch = excluded.hook_pre_launch, hook_wrapper = excluded.hook_wrapper,
                hook_post_exit = excluded.hook_post_exit
            "#,
            sql.id, sql.name, sql.icon_path, sql.install_stage, sql.game_version, sql.loader,
            sql.loader_version_json, sql.java_path, sql.launch_args_json, sql.env_vars_json,
            sql.memory_maximum, sql.force_fullscreen, sql.window_width, sql.window_height,
            sql.created_at, sql.modified_at, sql.last_played_at, sql.time_played,
            sql.recent_time_played, sql.hook_pre_launch, sql.hook_wrapper, sql.hook_post_exit
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let instance_id = instance.id().to_owned();

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

        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }

    async fn remove(&self, id: &str) -> Result<(), InstanceError> {
        sqlx::query!("DELETE FROM instances WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        Ok(())
    }
}
