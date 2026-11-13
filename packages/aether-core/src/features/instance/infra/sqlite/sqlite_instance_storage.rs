use crate::features::instance::{
    BrokenInstanceReason, BrokenInstanceSource, BrokenRecord, Instance, InstanceChange,
    InstanceError, InstanceId, InstanceStorage,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Acquire, Sqlite, SqliteConnection, SqlitePool, Transaction};
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
            .map_err(InstanceError::storage)
    }

    /// Writes the instances in one transaction, each under its own savepoint: an instance that
    /// cannot be stored is left out and returned with its error, the rest are kept. A failure of
    /// the transaction itself (disk full, locked database) is an error: nothing is stored then.
    pub async fn upsert_many<'a>(
        &self,
        instances: &'a [Instance],
    ) -> Result<Vec<(&'a Instance, InstanceError)>, InstanceError> {
        let mut tx = self.begin_write().await?;
        let mut failed = Vec::new();
        for instance in instances {
            let mut savepoint = tx.begin().await.map_err(InstanceError::storage)?;
            match write_instance(&mut savepoint, instance).await {
                Ok(()) => savepoint.commit().await.map_err(InstanceError::storage)?,
                Err(e) => failed.push((instance, e)),
            }
        }
        tx.commit().await.map_err(InstanceError::storage)?;
        Ok(failed)
    }

    /// A storage over a fresh in-memory database with every migration applied.
    #[cfg(test)]
    pub async fn in_memory() -> Self {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);

        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to create in-memory SQLite pool");

        crate::core::migrations::run_migrations(&pool)
            .await
            .unwrap();

        Self::new(pool)
    }

    #[cfg(test)]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// What one pass over the `instances` table found.
#[derive(Default)]
struct Scan {
    healthy: Vec<Instance>,
    broken: Vec<(String, BrokenInstanceReason)>,
}

/// The timestamps are read as text and parsed per row: one that does not parse makes its
/// own row broken, where a typed column would fail the read of the whole table.
fn parse_timestamp(value: &str, column: &str) -> Result<DateTime<Utc>, BrokenInstanceReason> {
    // The forms the typed decode of `get` takes too, as far as a hand-edited value gets.
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .or_else(|_| {
            [
                "%Y-%m-%d %H:%M:%S%.f",
                "%Y-%m-%dT%H:%M:%S%.f",
                "%Y-%m-%d %H:%M",
            ]
            .iter()
            .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
            .map(|naive| naive.and_utc())
            .ok_or(())
        })
        .map_err(|()| BrokenInstanceReason::InvalidColumn {
            column: column.to_owned(),
        })
}

type Timestamps = (DateTime<Utc>, DateTime<Utc>, Option<DateTime<Utc>>);

fn parse_timestamps(
    created: &str,
    modified: &str,
    last_played: Option<&str>,
) -> Result<Timestamps, BrokenInstanceReason> {
    Ok((
        parse_timestamp(created, "created_at")?,
        parse_timestamp(modified, "modified_at")?,
        last_played
            .map(|value| parse_timestamp(value, "last_played_at"))
            .transpose()?,
    ))
}

impl SqliteInstanceStorage {
    #[allow(clippy::too_many_lines)]
    async fn scan(&self) -> Result<Scan, InstanceError> {
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
                i.created_at as "created_at: String", 
                i.modified_at as "modified_at: String", 
                i.last_played_at as "last_played_at?: String", 
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
        .map_err(InstanceError::storage)?;

        let mut scan = Scan::default();
        for row in rows {
            let row_id = row.id.clone();
            let timestamps = parse_timestamps(
                &row.created_at,
                &row.modified_at,
                row.last_played_at.as_deref(),
            );
            let (created_at, modified_at, last_played_at) = match timestamps {
                Ok(parsed) => parsed,
                Err(reason) => {
                    warn!("Failed to read instance {row_id:?}: {reason}");
                    scan.broken.push((row_id, reason));
                    continue;
                }
            };

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
                created_at,
                modified_at,
                last_played_at,
                time_played: row.time_played,
                recent_time_played: row.recent_time_played,
                override_hooks: row.override_hooks,
                hook_pre_launch: row.hook_pre_launch,
                hook_wrapper: row.hook_wrapper,
                hook_post_exit: row.hook_post_exit,
            };

            let mut instance = match sql_instance.into_instance() {
                Ok(instance) => instance,
                Err(reason) => {
                    warn!("Failed to read instance {row_id:?}: {reason}");
                    scan.broken.push((row_id, reason));
                    continue;
                }
            };

            if let (Some(pid), Some(mid), Some(vid)) = (
                row.pack_provider_id,
                row.pack_modpack_id,
                row.pack_version_id,
            ) {
                instance.pack_info = SqlPackInfo {
                    provider_id: pid,
                    modpack_id: mid,
                    version_id: vid,
                }
                .into_pack_info();
            }

            scan.healthy.push(instance);
        }
        Ok(scan)
    }
}

#[async_trait]
impl InstanceStorage for SqliteInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.scan().await?.healthy)
    }

    async fn stored_ids(&self) -> Result<Vec<String>, InstanceError> {
        sqlx::query_scalar!("SELECT id FROM instances")
            .fetch_all(&self.pool)
            .await
            .map_err(InstanceError::storage)
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        let mut conn = self.pool.acquire().await.map_err(InstanceError::storage)?;
        read_instance(&mut conn, id).await
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut tx = self.begin_write().await?;
        write_instance(&mut tx, instance).await?;
        tx.commit().await.map_err(InstanceError::storage)
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
        tx.commit().await.map_err(InstanceError::storage)?;
        Ok(instance)
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let mut tx = self.begin_write().await?;
        sqlx::query!("DELETE FROM instances WHERE id = ?", id.as_str())
            .execute(&mut *tx)
            .await
            .map_err(InstanceError::storage)?;
        tx.commit().await.map_err(InstanceError::storage)
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
    .map_err(InstanceError::storage)?;

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
        instance.pack_info = SqlPackInfo {
            provider_id: pid,
            modpack_id: mid,
            version_id: vid,
        }
        .into_pack_info();
    }

    Ok(instance)
}

#[allow(clippy::too_many_lines)]
async fn write_instance(
    tx: &mut SqliteConnection,
    instance: &Instance,
) -> Result<(), InstanceError> {
    let sql = SqlInstance::try_from(instance.clone())?;

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
        .map_err(InstanceError::storage)?;

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
        .map_err(InstanceError::storage)?;

    Ok(())
}

#[async_trait]
impl BrokenInstanceSource for SqliteInstanceStorage {
    async fn list_broken(&self) -> Result<Vec<BrokenRecord>, InstanceError> {
        Ok(self
            .scan()
            .await?
            .broken
            .into_iter()
            .map(|(id, reason)| BrokenRecord { id, reason })
            .collect())
    }

    async fn remove_record(&self, id: &str) -> Result<(), InstanceError> {
        sqlx::query("DELETE FROM instances WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(InstanceError::storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::InstanceName;
    use crate::features::minecraft::GameVersion;
    use crate::features::{instance::InstanceBuilder, minecraft::ModLoader};

    fn instance(loader: ModLoader) -> Instance {
        InstanceBuilder::new(
            InstanceId::try_from("a").unwrap(),
            InstanceName::parse("a").unwrap(),
            GameVersion::from_static("1.21.1"),
            loader,
        )
        .build()
    }

    #[tokio::test]
    async fn should_round_trip_an_instance_keeping_the_stored_enum_strings() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage
            .upsert(&instance(ModLoader::NeoForge))
            .await
            .unwrap();

        let (loader, stage): (String, String) =
            sqlx::query_as("SELECT loader, install_stage FROM instances WHERE id = 'a'")
                .fetch_one(storage.pool())
                .await
                .unwrap();
        let read = storage
            .get(&InstanceId::try_from("a").unwrap())
            .await
            .unwrap();

        assert_eq!(loader, "neo_forge");
        assert_eq!(stage, "not_installed");
        assert_eq!(read.snapshot().loader, ModLoader::NeoForge);
    }

    #[tokio::test]
    async fn should_fail_to_read_an_instance_with_corrupt_launch_args() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query("UPDATE instances SET launch_args_json = 'not json' WHERE id = 'a'")
            .execute(storage.pool())
            .await
            .unwrap();

        let result = storage.get(&InstanceId::try_from("a").unwrap()).await;

        assert!(
            matches!(result, Err(InstanceError::InvalidRecord { reason: crate::features::instance::BrokenInstanceReason::InvalidColumn { ref column }, .. }) if column == "launch_args_json")
        );
    }

    #[tokio::test]
    async fn should_fail_to_read_an_instance_with_corrupt_env_vars() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query("UPDATE instances SET env_vars_json = '{' WHERE id = 'a'")
            .execute(storage.pool())
            .await
            .unwrap();

        let result = storage.get(&InstanceId::try_from("a").unwrap()).await;

        assert!(
            matches!(result, Err(InstanceError::InvalidRecord { reason: crate::features::instance::BrokenInstanceReason::InvalidColumn { ref column }, .. }) if column == "env_vars_json")
        );
    }

    #[tokio::test]
    async fn should_fall_back_to_the_id_when_the_stored_name_is_blank() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query("UPDATE instances SET name = '  ' WHERE id = 'a'")
            .execute(storage.pool())
            .await
            .unwrap();

        let result = storage.get(&InstanceId::try_from("a").unwrap()).await;

        assert_eq!(result.unwrap().name().as_str(), "a");
    }

    /// A row that cannot be read is told apart from the rest, with the reason, and the rest
    /// still lists: the broken one is not just missing.
    async fn broken(storage: &SqliteInstanceStorage) -> Vec<(String, BrokenInstanceReason)> {
        storage
            .list_broken()
            .await
            .unwrap()
            .into_iter()
            .map(|record| (record.id, record.reason))
            .collect()
    }

    #[tokio::test]
    async fn should_report_the_row_as_broken_when_its_game_version_is_invalid() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query("UPDATE instances SET game_version = '../x' WHERE id = 'a'")
            .execute(storage.pool())
            .await
            .unwrap();

        assert_eq!(
            broken(&storage).await,
            [(
                "a".to_owned(),
                BrokenInstanceReason::InvalidGameVersion {
                    value: "../x".to_owned()
                }
            )]
        );
    }

    #[tokio::test]
    async fn should_keep_listing_the_rest_when_a_timestamp_cannot_be_read() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        let other = InstanceBuilder::new(
            InstanceId::try_from("b").unwrap(),
            InstanceName::parse("b").unwrap(),
            GameVersion::from_static("1.21.1"),
            ModLoader::Vanilla,
        )
        .build();
        storage.upsert(&other).await.unwrap();
        sqlx::query("UPDATE instances SET created_at = 'garbage' WHERE id = 'a'")
            .execute(storage.pool())
            .await
            .unwrap();

        let listed = storage.list().await.unwrap();

        assert_eq!(listed.len(), 1, "the other instance still lists");
        assert_eq!(
            broken(&storage).await,
            [(
                "a".to_owned(),
                BrokenInstanceReason::InvalidColumn {
                    column: "created_at".to_owned()
                }
            )]
        );
    }

    #[tokio::test]
    async fn should_report_the_row_as_broken_when_its_id_is_not_a_directory_name() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query("UPDATE instances SET id = '../x'")
            .execute(storage.pool())
            .await
            .unwrap();

        assert_eq!(
            broken(&storage).await,
            [("../x".to_owned(), BrokenInstanceReason::InvalidId)]
        );
        assert_eq!(storage.stored_ids().await.unwrap(), ["../x"]);
    }

    #[tokio::test]
    async fn should_fail_to_read_an_instance_when_its_stored_game_version_is_invalid() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query("UPDATE instances SET game_version = '../x' WHERE id = 'a'")
            .execute(storage.pool())
            .await
            .unwrap();

        let result = storage.get(&InstanceId::try_from("a").unwrap()).await;
        let listed = storage.list().await.unwrap();

        assert!(
            matches!(
                result,
                Err(InstanceError::InvalidRecord {
                    reason: BrokenInstanceReason::InvalidGameVersion { .. },
                    ..
                })
            ),
            "{result:?}"
        );
        assert!(
            listed.is_empty(),
            "an unreadable instance is left out of the list"
        );
    }

    #[tokio::test]
    async fn should_drop_only_the_loader_version_when_its_stored_id_is_invalid() {
        let storage = SqliteInstanceStorage::in_memory().await;
        storage.upsert(&instance(ModLoader::Fabric)).await.unwrap();
        sqlx::query(r#"UPDATE instances SET loader_version_json = '"0.16 .5"' WHERE id = 'a'"#)
            .execute(storage.pool())
            .await
            .unwrap();

        let read = storage
            .get(&InstanceId::try_from("a").unwrap())
            .await
            .unwrap();

        assert_eq!(read.snapshot().loader, ModLoader::Fabric);
        assert_eq!(read.snapshot().loader_version, None);
    }
}
