use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::features::process::{ProcessRecord, ProcessRecordStorage};
use crate::shared::error::DatabaseError;

use super::sql_process_record::{SqlProcessRecord, format_timestamp};

pub struct SqliteProcessRecordStorage {
    pool: SqlitePool,
}

impl SqliteProcessRecordStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProcessRecordStorage for SqliteProcessRecordStorage {
    async fn insert(&self, record: &ProcessRecord) -> Result<(), DatabaseError> {
        let row = SqlProcessRecord::from(record);
        sqlx::query!(
            "INSERT INTO game_processes (
                id, instance_id, pid, start_stamp, boot_id, started_at, kill_scope,
                post_exit_command, playtime_credited_until, owner_pid, owner_start_stamp,
                owner_boot_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            row.id,
            row.instance_id,
            row.pid,
            row.start_stamp,
            row.boot_id,
            row.started_at,
            row.kill_scope,
            row.post_exit_command,
            row.playtime_credited_until,
            row.owner_pid,
            row.owner_start_stamp,
            row.owner_boot_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let id = id.to_string();
        let removed = sqlx::query!("DELETE FROM game_processes WHERE id = ?", id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(removed > 0)
    }

    async fn list(&self) -> Result<Vec<ProcessRecord>, DatabaseError> {
        let rows = sqlx::query_as!(
            SqlProcessRecord,
            r#"SELECT
                id, instance_id, pid, start_stamp, boot_id,
                started_at AS "started_at: String", kill_scope, post_exit_command,
                playtime_credited_until AS "playtime_credited_until: String",
                owner_pid, owner_start_stamp, owner_boot_id
            FROM game_processes"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(SqlProcessRecord::into_record)
            .collect())
    }

    async fn list_for_instance(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Vec<ProcessRecord>, DatabaseError> {
        let instance_id = instance_id.as_str();
        let rows = sqlx::query_as!(
            SqlProcessRecord,
            r#"SELECT
                id, instance_id, pid, start_stamp, boot_id,
                started_at AS "started_at: String", kill_scope, post_exit_command,
                playtime_credited_until AS "playtime_credited_until: String",
                owner_pid, owner_start_stamp, owner_boot_id
            FROM game_processes
            WHERE instance_id = ?"#,
            instance_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(SqlProcessRecord::into_record)
            .collect())
    }

    async fn touch_playtime(&self, id: Uuid, until: DateTime<Utc>) -> Result<(), DatabaseError> {
        let id = id.to_string();
        let until = format_timestamp(until);
        sqlx::query!(
            "UPDATE game_processes SET playtime_credited_until = ? WHERE id = ?",
            until,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;

    use super::*;
    use crate::features::instance::infra::SqliteInstanceStorage;
    use crate::features::instance::{InstanceBuilder, InstanceName, InstanceStorage};
    use crate::features::minecraft::{GameVersion, ModLoader};
    use crate::features::process::{KillScope, OsProcessIdentity};

    /// Over a fresh database with foreign keys enforced, as the app opens it.
    async fn storages() -> (SqliteProcessRecordStorage, SqliteInstanceStorage) {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(true);
        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        crate::core::migrations::run_migrations(&pool)
            .await
            .unwrap();
        (
            SqliteProcessRecordStorage::new(pool.clone()),
            SqliteInstanceStorage::new(pool),
        )
    }

    async fn add_instance(instances: &SqliteInstanceStorage, id: &str) -> InstanceId {
        let id = InstanceId::try_from(id).unwrap();
        let instance = InstanceBuilder::new(
            id.clone(),
            InstanceName::parse("Test Instance").unwrap(),
            GameVersion::from_static("1.20.1"),
            ModLoader::Vanilla,
        )
        .build();
        instances.upsert(&instance).await.unwrap();
        id
    }

    fn record(instance_id: &InstanceId) -> ProcessRecord {
        let started_at = Utc::now();
        ProcessRecord {
            id: Uuid::new_v4(),
            instance_id: instance_id.clone(),
            process: OsProcessIdentity::new(4242, 987_654_321, "boot-a".to_owned()),
            started_at,
            kill_scope: KillScope::Tree,
            post_exit_command: "backup.sh --all".to_owned(),
            playtime_credited_until: started_at,
            owner: OsProcessIdentity::new(17, 123, "boot-a".to_owned()),
        }
    }

    #[tokio::test]
    async fn should_read_back_the_record_as_it_was_inserted() {
        let (records, instances) = storages().await;
        let instance_id = add_instance(&instances, "a").await;
        let record = record(&instance_id);

        records.insert(&record).await.unwrap();

        assert_eq!(records.list().await.unwrap(), [record]);
    }

    #[tokio::test]
    async fn should_list_only_the_records_of_the_instance_when_asked_for_one() {
        let (records, instances) = storages().await;
        let a = add_instance(&instances, "a").await;
        let b = add_instance(&instances, "b").await;
        let of_a = record(&a);
        records.insert(&of_a).await.unwrap();
        records.insert(&record(&b)).await.unwrap();

        assert_eq!(records.list_for_instance(&a).await.unwrap(), [of_a]);
    }

    #[tokio::test]
    async fn should_forget_the_record_when_it_is_removed() {
        let (records, instances) = storages().await;
        let instance_id = add_instance(&instances, "a").await;
        let removed = record(&instance_id);
        let kept = record(&instance_id);
        records.insert(&removed).await.unwrap();
        records.insert(&kept).await.unwrap();

        let was_there = records.remove(removed.id).await.unwrap();

        assert!(was_there);
        assert_eq!(records.list().await.unwrap(), [kept]);
    }

    #[tokio::test]
    async fn should_move_only_the_playtime_mark_when_it_is_touched() {
        let (records, instances) = storages().await;
        let instance_id = add_instance(&instances, "a").await;
        let mut record = record(&instance_id);
        records.insert(&record).await.unwrap();
        let until = record.started_at + chrono::Duration::seconds(60);

        records.touch_playtime(record.id, until).await.unwrap();

        record.playtime_credited_until = until;
        assert_eq!(records.list().await.unwrap(), [record]);
    }

    #[tokio::test]
    async fn should_report_nothing_removed_and_not_fail_when_the_record_is_missing() {
        let (records, _) = storages().await;

        let was_there = records.remove(Uuid::new_v4()).await.unwrap();
        assert!(!was_there);
        records
            .touch_playtime(Uuid::new_v4(), Utc::now())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_drop_the_records_of_an_instance_when_it_is_deleted() {
        let (records, instances) = storages().await;
        let deleted = add_instance(&instances, "a").await;
        let other = add_instance(&instances, "b").await;
        records.insert(&record(&deleted)).await.unwrap();
        let kept = record(&other);
        records.insert(&kept).await.unwrap();

        instances.remove(&deleted).await.unwrap();

        assert_eq!(records.list().await.unwrap(), [kept]);
    }
}
