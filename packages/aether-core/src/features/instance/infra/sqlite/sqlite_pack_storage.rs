use crate::features::instance::{
    ContentFileUpdateInfo, FileSide, InstanceError, InstanceId, Pack, PackEntry, PackFile,
    PackStorage, PlanOrigin, ProviderId,
    infra::{
        legacy_provider_id::{legacy_provider_id_string, parse_stored_provider_id},
        stored_side::parse_stored_side,
    },
};
use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;

pub struct SqlitePackStorage {
    pool: SqlitePool,
}

struct PackFileRow {
    content_path: String,
    file_name: String,
    name: Option<String>,
    hash: String,
    side: Option<String>,
    update_provider_id: Option<String>,
    installed_by_provider_id: Option<String>,
    installed_by_content_id: Option<String>,
}

struct PackFileUpdateRow {
    content_path: String,
    provider_id: String,
    content_id: String,
    version_id: String,
}

impl PackFileRow {
    fn into_pack_file(self, updates: Vec<PackFileUpdateRow>) -> PackFile {
        let mut update_map = HashMap::new();
        // An update that no longer parses is skipped rather than failing the read, so one bad row
        // does not hide the rest of the pack.
        for u in updates {
            match parse_stored_provider_id(&u.provider_id) {
                Ok(provider_id) => {
                    update_map.insert(
                        provider_id,
                        ContentFileUpdateInfo {
                            content_id: u.content_id,
                            version: u.version_id,
                        },
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        "Skipping update \"{}\" of pack file \"{}\": {err}",
                        u.provider_id,
                        self.content_path
                    );
                }
            }
        }

        // An owner that no longer parses is dropped rather than failing the read: a file without
        // one is never removed by a plan, which is the safe side.
        let installed_by = match (self.installed_by_provider_id, self.installed_by_content_id) {
            (Some(provider_id), Some(content_id)) => match parse_stored_provider_id(&provider_id) {
                Ok(provider_id) => Some(PlanOrigin {
                    provider_id,
                    content_id,
                }),
                Err(err) => {
                    tracing::warn!(
                        "Ignoring the owner of pack file \"{}\": {err}",
                        self.content_path
                    );
                    None
                }
            },
            _ => None,
        };

        let side = parse_stored_side(&self.content_path, self.side.as_deref());

        // The `pack_files` table has no columns for `download` and `option`, so a file read back
        // from SQLite has neither.
        PackFile {
            file_name: self.file_name,
            name: self.name,
            hash: self.hash,
            download: None,
            option: None,
            side,
            update_provider_id: self
                .update_provider_id
                .and_then(|p| parse_stored_provider_id(&p).ok()),
            update: if update_map.is_empty() {
                None
            } else {
                Some(update_map)
            },
            installed_by,
        }
    }
}

impl SqlitePackStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// A storage over a fresh in-memory database with every migration applied. Foreign keys are
    /// off: no `instances` row backs the packs.
    #[cfg(test)]
    pub async fn in_memory() -> Self {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(false);

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

    /// Whether the instance has a pack row. The row is created with the first file and stays
    /// when the last one is removed, so it tells a pack that was emptied from one never stored.
    pub async fn has_pack(&self, instance_id: &InstanceId) -> Result<bool, InstanceError> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM packs WHERE instance_id = ?")
            .bind(instance_id.as_str())
            .fetch_one(&self.pool)
            .await
            .map(|count| count > 0)
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }

    // Helper to internalize update logic for reuse in transactions
    async fn upsert_file_internal(
        tx: &mut Transaction<'_, Sqlite>,
        instance_id: &InstanceId,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        // Ensure pack exists
        sqlx::query!(
            "INSERT OR IGNORE INTO packs (instance_id) VALUES (?)",
            instance_id.as_str()
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let update_provider_id = pack_file
            .update_provider_id
            .as_ref()
            .map(std::string::ToString::to_string);

        let installed_by_provider_id = pack_file
            .installed_by
            .as_ref()
            .map(|origin| origin.provider_id.to_string());
        let installed_by_content_id = pack_file
            .installed_by
            .as_ref()
            .map(|origin| origin.content_id.as_str());

        sqlx::query!(
            r#"
            INSERT INTO pack_files (instance_id, content_path, file_name, name, hash, side, update_provider_id, installed_by_provider_id, installed_by_content_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(instance_id, content_path) DO UPDATE SET
                file_name = excluded.file_name,
                name = excluded.name,
                hash = excluded.hash,
                side = excluded.side,
                update_provider_id = excluded.update_provider_id,
                installed_by_provider_id = excluded.installed_by_provider_id,
                installed_by_content_id = excluded.installed_by_content_id
            "#,
            instance_id.as_str(),
            content_path,
            pack_file.file_name,
            pack_file.name,
            pack_file.hash,
            pack_file.side.map(FileSide::as_str),
            update_provider_id,
            installed_by_provider_id,
            installed_by_content_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        // Cleanup old updates
        sqlx::query!(
            "DELETE FROM pack_file_updates WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        // Insert new updates
        if let Some(updates) = &pack_file.update {
            for (pid, info) in updates {
                let pid_str = pid.to_string();
                sqlx::query!(
                    "INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?, ?, ?, ?, ?)",
                    instance_id.as_str(),
                    content_path,
                    pid_str,
                    info.content_id,
                    info.version
                )
                .execute(&mut **tx)
                .await
                .map_err(|e| InstanceError::Storage(e.to_string()))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl PackStorage for SqlitePackStorage {
    async fn get_pack(&self, instance_id: &InstanceId) -> Result<Pack, InstanceError> {
        let files = sqlx::query!(
            "SELECT content_path FROM pack_files WHERE instance_id = ? ORDER BY content_path ASC",
            instance_id.as_str()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        Ok(Pack {
            files: files
                .into_iter()
                .map(|r| PackEntry {
                    file: r.content_path,
                })
                .collect(),
        })
    }

    async fn update_pack(
        &self,
        _instance_id: &InstanceId,
        _pack: &Pack,
    ) -> Result<(), InstanceError> {
        // In SQL, the "pack" is just a collection of files.
        // Individual file updates manage this state.
        Ok(())
    }

    async fn get_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
    ) -> Result<PackFile, InstanceError> {
        let row = sqlx::query_as!(
            PackFileRow,
            "SELECT content_path, file_name, name, hash, side, update_provider_id, installed_by_provider_id, installed_by_content_id FROM pack_files WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?
        .ok_or_else(|| InstanceError::Storage(format!("Pack file not found: {content_path}")))?;

        let updates = sqlx::query_as!(
            PackFileUpdateRow,
            "SELECT content_path, provider_id, content_id, version_id FROM pack_file_updates WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        Ok(row.into_pack_file(updates))
    }

    async fn get_pack_files(
        &self,
        instance_id: &InstanceId,
    ) -> Result<HashMap<String, PackFile>, InstanceError> {
        let rows = sqlx::query_as!(
            PackFileRow,
            "SELECT content_path, file_name, name, hash, side, update_provider_id, installed_by_provider_id, installed_by_content_id FROM pack_files WHERE instance_id = ?",
            instance_id.as_str()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let all_updates = sqlx::query_as!(
            PackFileUpdateRow,
            "SELECT content_path, provider_id, content_id, version_id FROM pack_file_updates WHERE instance_id = ?",
            instance_id.as_str()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let mut updates: HashMap<String, Vec<PackFileUpdateRow>> = HashMap::new();
        for update in all_updates {
            updates
                .entry(update.content_path.clone())
                .or_default()
                .push(update);
        }

        Ok(rows
            .into_iter()
            .map(|row| {
                let file_updates = updates.remove(&row.content_path).unwrap_or_default();
                (row.content_path.clone(), row.into_pack_file(file_updates))
            })
            .collect())
    }

    async fn find_by_provider_id(
        &self,
        instance_id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &str,
    ) -> Result<Option<PackFile>, InstanceError> {
        let pid = provider_id.to_string();
        // Rows are not rewritten: a file saved under a provider's earlier id is matched by that
        // id too, so both ids are queried.
        let legacy_pid = legacy_provider_id_string(provider_id).unwrap_or_else(|| pid.clone());
        let row = sqlx::query!(
            "SELECT content_path FROM pack_file_updates WHERE instance_id = ? AND provider_id IN (?, ?) AND content_id = ? LIMIT 1",
            instance_id.as_str(),
            pid,
            legacy_pid,
            content_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        if let Some(r) = row {
            return Ok(Some(
                self.get_pack_file(instance_id, &r.content_path).await?,
            ));
        }

        Ok(None)
    }

    async fn update_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        Self::upsert_file_internal(&mut tx, instance_id, content_path, pack_file).await?;
        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }

    async fn update_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
        pack_files: &[PackFile],
    ) -> Result<(), InstanceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;

        for (path, file) in content_paths.iter().zip(pack_files) {
            Self::upsert_file_internal(&mut tx, instance_id, path, file).await?;
        }

        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }

    async fn remove_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
    ) -> Result<(), InstanceError> {
        sqlx::query!(
            "DELETE FROM pack_files WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .execute(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn remove_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;

        for path in content_paths {
            sqlx::query!(
                "DELETE FROM pack_files WHERE instance_id = ? AND content_path = ?",
                instance_id.as_str(),
                path
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }

    async fn remove_all_for_instance(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;

        sqlx::query!(
            "DELETE FROM pack_file_updates WHERE instance_id = ?",
            instance_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        sqlx::query!(
            "DELETE FROM pack_files WHERE instance_id = ?",
            instance_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        sqlx::query!(
            "DELETE FROM packs WHERE instance_id = ?",
            instance_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_skip_update_when_its_provider_id_is_malformed() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let good: ProviderId = "plugin#cap".parse().unwrap();
        let file = PackFile {
            update: Some(HashMap::from([(
                good.clone(),
                ContentFileUpdateInfo {
                    content_id: "c1".into(),
                    version: "v1".into(),
                },
            )])),
            ..PackFile::from_hash("a.jar".into(), "h".into())
        };
        storage
            .update_pack_file(&id, "mods/a.jar", &file)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES ('inst', 'mods/a.jar', 'malformed', 'c2', 'v2')",
        )
        .execute(&storage.pool)
        .await
        .unwrap();

        let single = storage.get_pack_file(&id, "mods/a.jar").await.unwrap();
        let all = storage.get_pack_files(&id).await.unwrap();

        for read in [&single, &all["mods/a.jar"]] {
            let updates = read.update.as_ref().unwrap();
            assert_eq!(updates.len(), 1);
            assert_eq!(updates[&good].content_id, "c1");
        }
    }
}
