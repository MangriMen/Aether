use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::features::process::{OsProcessIdentity, ProcessRecord, ProcessRecordStorage};
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

    async fn take_over(
        &self,
        id: Uuid,
        from: &OsProcessIdentity,
        to: &OsProcessIdentity,
    ) -> Result<bool, DatabaseError> {
        let id = id.to_string();
        let (from_pid, from_stamp) = (i64::from(from.pid()), from.start_stamp().cast_signed());
        let (to_pid, to_stamp) = (i64::from(to.pid()), to.start_stamp().cast_signed());
        let (from_boot, to_boot) = (from.boot_id(), to.boot_id());
        // One statement, so that of two launchers taking over the same game only the first
        // finds the owner it read.
        let taken = sqlx::query!(
            "UPDATE game_processes
            SET owner_pid = ?, owner_start_stamp = ?, owner_boot_id = ?
            WHERE id = ? AND owner_pid = ? AND owner_start_stamp = ? AND owner_boot_id = ?",
            to_pid,
            to_stamp,
            to_boot,
            id,
            from_pid,
            from_stamp,
            from_boot,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(taken == 1)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;

    use super::*;
    use crate::features::instance::infra::SqliteInstanceStorage;
    use crate::features::instance::{InstanceBuilder, InstanceName, InstanceStorage};
    use crate::features::minecraft::{GameVersion, ModLoader};
    use crate::features::process::KillScope;

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
    async fn should_take_over_the_record_once_when_two_launchers_take_it_over_from_the_same_owner()
    {
        let (records, instances) = storages().await;
        let instance_id = add_instance(&instances, "a").await;
        let mut record = record(&instance_id);
        records.insert(&record).await.unwrap();
        let first = OsProcessIdentity::new(30, 300, "boot-a".to_owned());
        let second = OsProcessIdentity::new(31, 310, "boot-a".to_owned());

        let first_took = records
            .take_over(record.id, &record.owner, &first)
            .await
            .unwrap();
        let second_took = records
            .take_over(record.id, &record.owner, &second)
            .await
            .unwrap();

        assert!(first_took);
        assert!(!second_took);
        record.owner = first;
        assert_eq!(records.list().await.unwrap(), [record]);
    }

    #[tokio::test]
    async fn should_not_take_over_the_record_when_the_owner_differs_in_its_boot_only() {
        let (records, instances) = storages().await;
        let instance_id = add_instance(&instances, "a").await;
        let record = record(&instance_id);
        records.insert(&record).await.unwrap();
        let other_boot = OsProcessIdentity::new(
            record.owner.pid(),
            record.owner.start_stamp(),
            "boot-b".to_owned(),
        );
        let launcher = OsProcessIdentity::new(30, 300, "boot-a".to_owned());

        let took = records
            .take_over(record.id, &other_boot, &launcher)
            .await
            .unwrap();
        let missing = records
            .take_over(Uuid::new_v4(), &record.owner, &launcher)
            .await
            .unwrap();

        assert!(!took);
        assert!(!missing);
        assert_eq!(records.list().await.unwrap(), [record]);
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
