use crate::features::instance::{
    ContentFileUpdateInfo, ContentId, ContentPath, FileSide, InstanceError, InstanceId, Pack,
    PackEntry, PackFile, PackStorage, PlanOrigin, ProviderId,
    infra::{
        legacy_provider_id::{legacy_provider_id_string, parse_stored_provider_id},
        stored_content_id::{parse_stored_content_id, parse_stored_version},
        stored_content_path::parse_stored_content_path,
        stored_hash::parse_stored_hash,
        stored_side::parse_stored_side,
    },
};
use crate::shared::hash::{FileHash, InvalidFileHash};
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
    /// The record, or why its stored hash is not one.
    fn into_checked_pack_file(
        self,
        updates: Vec<PackFileUpdateRow>,
    ) -> Result<PackFile, InvalidFileHash> {
        let hash = parse_stored_hash(&self.content_path, &self.hash)?;
        Ok(self.into_pack_file(updates, hash))
    }

    /// The record with `hash` in place of the stored one.
    fn into_pack_file(self, updates: Vec<PackFileUpdateRow>, hash: FileHash) -> PackFile {
        let mut update_map = HashMap::new();
        // An update that no longer parses is skipped rather than failing the read, so one bad row
        // does not hide the rest of the pack.
        for u in updates {
            match parse_stored_provider_id(&u.provider_id) {
                Ok(provider_id) => {
                    let owner = format!(
                        "update \"{}\" of pack file \"{}\"",
                        u.provider_id, self.content_path
                    );
                    let Some(content_id) = parse_stored_content_id(&owner, u.content_id) else {
                        continue;
                    };
                    let version = parse_stored_version(&owner, u.version_id);
                    update_map.insert(
                        provider_id,
                        ContentFileUpdateInfo {
                            content_id,
                            version,
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
                Ok(provider_id) => {
                    let owner = format!("the owner of pack file \"{}\"", self.content_path);
                    parse_stored_content_id(&owner, content_id).map(|content_id| PlanOrigin {
                        provider_id,
                        content_id,
                    })
                }
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
            hash,
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

    /// The row of a file and its updates: `None` when there is none.
    async fn load_rows(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<Option<(PackFileRow, Vec<PackFileUpdateRow>)>, InstanceError> {
        let content_path = content_path.as_str();
        let row = sqlx::query_as!(
            PackFileRow,
            "SELECT content_path, file_name, name, hash, side, update_provider_id, installed_by_provider_id, installed_by_content_id FROM pack_files WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(InstanceError::storage)?;
        let Some(row) = row else {
            return Ok(None);
        };

        let updates = sqlx::query_as!(
            PackFileUpdateRow,
            "SELECT content_path, provider_id, content_id, version_id FROM pack_file_updates WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .fetch_all(&self.pool)
        .await
        .map_err(InstanceError::storage)?;

        Ok(Some((row, updates)))
    }

    /// The record of a file: `None` when there is none, `Err` when its stored hash is invalid.
    async fn load_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<Option<Result<PackFile, InvalidFileHash>>, InstanceError> {
        Ok(self
            .load_rows(instance_id, content_path)
            .await?
            .map(|(row, updates)| row.into_checked_pack_file(updates)))
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

    /// The pool under the storage, for a test to plant rows the storage would not write.
    #[cfg(test)]
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Whether the instance has a pack row. The row is created with the first file and stays
    /// when the last one is removed, so it tells a pack that was emptied from one never stored.
    pub async fn has_pack(&self, instance_id: &InstanceId) -> Result<bool, InstanceError> {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM packs WHERE instance_id = ?",
            instance_id.as_str()
        )
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(InstanceError::storage)
    }

    /// Writes the file only when there is no row for the path; a row that is there, with its
    /// owner and updates, is left as it is.
    async fn insert_file_if_absent_internal(
        tx: &mut Transaction<'_, Sqlite>,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        let content_path = content_path.as_str();
        sqlx::query!(
            "INSERT OR IGNORE INTO packs (instance_id) VALUES (?)",
            instance_id.as_str()
        )
        .execute(&mut **tx)
        .await
        .map_err(InstanceError::storage)?;

        let hash = pack_file.hash.to_string();
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

        let inserted = sqlx::query!(
            r#"
            INSERT INTO pack_files (instance_id, content_path, file_name, name, hash, side, update_provider_id, installed_by_provider_id, installed_by_content_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(instance_id, content_path) DO NOTHING
            "#,
            instance_id.as_str(),
            content_path,
            pack_file.file_name,
            pack_file.name,
            hash,
            pack_file.side.map(FileSide::as_str),
            update_provider_id,
            installed_by_provider_id,
            installed_by_content_id
        )
        .execute(&mut **tx)
        .await
        .map_err(InstanceError::storage)?
        .rows_affected();
        if inserted == 0 {
            return Ok(());
        }

        for (pid, info) in pack_file.update.iter().flatten() {
            let provider_id = pid.to_string();
            sqlx::query!(
                "INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?, ?, ?, ?, ?)",
                instance_id.as_str(),
                content_path,
                provider_id,
                info.content_id.as_str(),
                info.version.as_ref().map_or("", |v| v.as_str())
            )
            .execute(&mut **tx)
            .await
            .map_err(InstanceError::storage)?;
        }
        Ok(())
    }

    // Helper to internalize update logic for reuse in transactions
    async fn upsert_file_internal(
        tx: &mut Transaction<'_, Sqlite>,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        let content_path = content_path.as_str();
        // Ensure pack exists
        sqlx::query!(
            "INSERT OR IGNORE INTO packs (instance_id) VALUES (?)",
            instance_id.as_str()
        )
        .execute(&mut **tx)
        .await
        .map_err(InstanceError::storage)?;

        let hash = pack_file.hash.to_string();
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
            hash,
            pack_file.side.map(FileSide::as_str),
            update_provider_id,
            installed_by_provider_id,
            installed_by_content_id
        )
        .execute(&mut **tx)
        .await
        .map_err(InstanceError::storage)?;

        // Cleanup old updates
        sqlx::query!(
            "DELETE FROM pack_file_updates WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .execute(&mut **tx)
        .await
        .map_err(InstanceError::storage)?;

        // Insert new updates
        if let Some(updates) = &pack_file.update {
            for (pid, info) in updates {
                let version = info.version.as_ref().map_or("", |v| v.as_str());
                let pid_str = pid.to_string();
                sqlx::query!(
                    "INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?, ?, ?, ?, ?)",
                    instance_id.as_str(),
                    content_path,
                    pid_str,
                    info.content_id.as_str(),
                    version
                )
                .execute(&mut **tx)
                .await
                .map_err(InstanceError::storage)?;
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
        .map_err(InstanceError::storage)?;

        Ok(Pack {
            files: files
                .into_iter()
                .filter_map(|r| {
                    parse_stored_content_path(instance_id, &r.content_path)
                        .map(|file| PackEntry { file })
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
        content_path: &ContentPath,
    ) -> Result<PackFile, InstanceError> {
        self.load_pack_file(instance_id, content_path)
            .await?
            .ok_or_else(|| InstanceError::PackFileNotFound {
                instance_id: instance_id.to_string(),
                content_path: content_path.to_string(),
            })?
            .map_err(InstanceError::storage)
    }

    async fn rehash_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        hash: &FileHash,
    ) -> Result<Option<PackFile>, InstanceError> {
        // One statement: a concurrent write to any other column or to the updates is never reverted.
        let hash = hash.to_string();
        let content_path_str = content_path.as_str();
        let changed = sqlx::query!(
            "UPDATE pack_files SET hash = ? WHERE instance_id = ? AND content_path = ?",
            hash,
            instance_id.as_str(),
            content_path_str
        )
        .execute(&self.pool)
        .await
        .map_err(InstanceError::storage)?
        .rows_affected();
        if changed == 0 {
            return Ok(None);
        }

        Ok(self
            .load_pack_file(instance_id, content_path)
            .await?
            .and_then(Result::ok))
    }

    async fn insert_pack_file_if_absent(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        pack_file: &PackFile,
    ) -> Result<PackFile, InstanceError> {
        let mut tx = self.pool.begin().await.map_err(InstanceError::storage)?;
        Self::insert_file_if_absent_internal(&mut tx, instance_id, content_path, pack_file).await?;
        tx.commit().await.map_err(InstanceError::storage)?;

        match self.load_pack_file(instance_id, content_path).await? {
            Some(Ok(stored)) => Ok(stored),
            // A row that exists but cannot be used keeps all but its hash.
            Some(Err(_)) => Ok(self
                .rehash_pack_file(instance_id, content_path, &pack_file.hash)
                .await?
                .unwrap_or_else(|| pack_file.clone())),
            None => Ok(pack_file.clone()),
        }
    }

    async fn find_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<Option<PackFile>, InstanceError> {
        Ok(self
            .load_pack_file(instance_id, content_path)
            .await?
            .and_then(Result::ok))
    }

    async fn get_pack_files(
        &self,
        instance_id: &InstanceId,
    ) -> Result<HashMap<ContentPath, PackFile>, InstanceError> {
        let rows = sqlx::query_as!(
            PackFileRow,
            "SELECT content_path, file_name, name, hash, side, update_provider_id, installed_by_provider_id, installed_by_content_id FROM pack_files WHERE instance_id = ?",
            instance_id.as_str()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(InstanceError::storage)?;

        let all_updates = sqlx::query_as!(
            PackFileUpdateRow,
            "SELECT content_path, provider_id, content_id, version_id FROM pack_file_updates WHERE instance_id = ?",
            instance_id.as_str()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(InstanceError::storage)?;

        let mut updates: HashMap<String, Vec<PackFileUpdateRow>> = HashMap::new();
        for update in all_updates {
            updates
                .entry(update.content_path.clone())
                .or_default()
                .push(update);
        }

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let content_path = parse_stored_content_path(instance_id, &row.content_path)?;
                let file_updates = updates.remove(&row.content_path).unwrap_or_default();
                let pack_file = row.into_checked_pack_file(file_updates).ok()?;
                Some((content_path, pack_file))
            })
            .collect())
    }

    async fn find_by_provider_id(
        &self,
        instance_id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &ContentId,
    ) -> Result<Option<PackFile>, InstanceError> {
        let pid = provider_id.to_string();
        // Rows are not rewritten: a file saved under a provider's earlier id is matched by that
        // id too, so both ids are queried.
        let legacy_pid = legacy_provider_id_string(provider_id).unwrap_or_else(|| pid.clone());
        let rows = sqlx::query!(
            "SELECT content_path FROM pack_file_updates WHERE instance_id = ? AND provider_id IN (?, ?) AND content_id = ? ORDER BY content_path ASC",
            instance_id.as_str(),
            pid,
            legacy_pid,
            content_id.as_str()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(InstanceError::storage)?;

        // A row whose path cannot be a content path matches nothing; the first one that can wins.
        if let Some(content_path) = rows
            .iter()
            .find_map(|row| parse_stored_content_path(instance_id, &row.content_path))
        {
            return Ok(Some(self.get_pack_file(instance_id, &content_path).await?));
        }

        Ok(None)
    }

    async fn update_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        let mut tx = self.pool.begin().await.map_err(InstanceError::storage)?;
        Self::upsert_file_internal(&mut tx, instance_id, content_path, pack_file).await?;
        tx.commit().await.map_err(InstanceError::storage)
    }

    async fn update_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
        pack_files: &[PackFile],
    ) -> Result<(), InstanceError> {
        let mut tx = self.pool.begin().await.map_err(InstanceError::storage)?;

        for (path, file) in content_paths.iter().zip(pack_files) {
            Self::upsert_file_internal(&mut tx, instance_id, path, file).await?;
        }

        tx.commit().await.map_err(InstanceError::storage)
    }

    async fn remove_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<(), InstanceError> {
        let content_path = content_path.as_str();
        sqlx::query!(
            "DELETE FROM pack_files WHERE instance_id = ? AND content_path = ?",
            instance_id.as_str(),
            content_path
        )
        .execute(&self.pool)
        .await
        .map_err(InstanceError::storage)?;
        Ok(())
    }

    async fn remove_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        let mut tx = self.pool.begin().await.map_err(InstanceError::storage)?;

        for path in content_paths {
            sqlx::query!(
                "DELETE FROM pack_files WHERE instance_id = ? AND content_path = ?",
                instance_id.as_str(),
                path.as_str()
            )
            .execute(&mut *tx)
            .await
            .map_err(InstanceError::storage)?;
        }

        tx.commit().await.map_err(InstanceError::storage)
    }

    async fn remove_all_for_instance(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let mut tx = self.pool.begin().await.map_err(InstanceError::storage)?;

        sqlx::query!(
            "DELETE FROM pack_file_updates WHERE instance_id = ?",
            instance_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(InstanceError::storage)?;

        sqlx::query!(
            "DELETE FROM pack_files WHERE instance_id = ?",
            instance_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(InstanceError::storage)?;

        sqlx::query!(
            "DELETE FROM packs WHERE instance_id = ?",
            instance_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(InstanceError::storage)?;

        tx.commit().await.map_err(InstanceError::storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::ContentVersionId;

    fn sha1_hash(c: char) -> crate::shared::hash::FileHash {
        crate::shared::hash::FileHash::new(
            crate::shared::hash::HashAlgorithm::Sha1,
            c.to_string().repeat(40),
        )
        .unwrap()
    }

    fn cp(path: &str) -> ContentPath {
        ContentPath::parse(path).unwrap()
    }

    #[tokio::test]
    async fn should_skip_update_when_its_provider_id_is_malformed() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let good: ProviderId = "plugin#cap".parse().unwrap();
        let file = PackFile {
            update: Some(HashMap::from([(
                good.clone(),
                ContentFileUpdateInfo {
                    content_id: ContentId::try_from("c1").unwrap(),
                    version: Some(ContentVersionId::try_from("v1").unwrap()),
                },
            )])),
            ..PackFile::from_hash("a.jar".into(), sha1_hash('a'))
        };
        storage
            .update_pack_file(&id, &cp("mods/a.jar"), &file)
            .await
            .unwrap();
        sqlx::query!("INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES ('inst', 'mods/a.jar', 'malformed', 'c2', 'v2')")
        .execute(&storage.pool)
        .await
        .unwrap();

        let single = storage.get_pack_file(&id, &cp("mods/a.jar")).await.unwrap();
        let all = storage.get_pack_files(&id).await.unwrap();

        for read in [&single, &all[&cp("mods/a.jar")]] {
            let updates = read.update.as_ref().unwrap();
            assert_eq!(updates.len(), 1);
            assert_eq!(
                updates[&good].content_id,
                ContentId::try_from("c1").unwrap()
            );
        }
    }

    #[tokio::test]
    async fn should_keep_an_update_without_a_version_when_its_stored_version_is_empty() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let provider: ProviderId = "plugin#cap".parse().unwrap();
        let file = PackFile {
            update: Some(HashMap::from([(
                provider.clone(),
                ContentFileUpdateInfo {
                    content_id: ContentId::try_from("c1").unwrap(),
                    version: None,
                },
            )])),
            ..PackFile::from_hash("a.jar".into(), sha1_hash('a'))
        };
        storage
            .update_pack_file(&id, &cp("mods/a.jar"), &file)
            .await
            .unwrap();

        let read = storage.get_pack_file(&id, &cp("mods/a.jar")).await.unwrap();

        let updates = read.update.unwrap();
        assert_eq!(updates[&provider].content_id.as_str(), "c1");
        assert_eq!(updates[&provider].version, None);
    }
    #[tokio::test]
    async fn should_drop_a_stored_path_that_is_not_a_content_path_when_reading() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        storage
            .update_pack_file(
                &id,
                &cp("mods/a.jar"),
                &PackFile::from_hash("a.jar".into(), sha1_hash('a')),
            )
            .await
            .unwrap();
        for bad in ["../x.jar", ".metadata/instance.json", "mods//b.jar"] {
            sqlx::query!("INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES ('inst', ?, 'x.jar', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa')", bad)
            .execute(&storage.pool)
            .await
            .unwrap();
        }

        let pack = storage.get_pack(&id).await.unwrap();
        let files = storage.get_pack_files(&id).await.unwrap();

        assert_eq!(pack.files.len(), 1);
        assert_eq!(pack.files[0].file, cp("mods/a.jar"));
        assert_eq!(files.len(), 1);
        assert!(files.contains_key(&cp("mods/a.jar")));
    }

    #[tokio::test]
    async fn should_find_the_valid_row_when_an_invalid_one_sorts_first() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let provider: ProviderId = "plugin#cap".parse().unwrap();
        let file = PackFile {
            update: Some(HashMap::from([(
                provider.clone(),
                ContentFileUpdateInfo {
                    content_id: ContentId::try_from("c1").unwrap(),
                    version: Some(ContentVersionId::try_from("v1").unwrap()),
                },
            )])),
            ..PackFile::from_hash("good.jar".into(), sha1_hash('a'))
        };
        storage
            .update_pack_file(&id, &cp("mods/good.jar"), &file)
            .await
            .unwrap();
        for bad in [".metadata/old.jar", "../old.jar"] {
            sqlx::query!("INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES ('inst', ?, 'old.jar', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa')", bad)
                .execute(&storage.pool)
                .await
                .unwrap();
            sqlx::query!("INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES ('inst', ?, 'plugin#cap', 'c1', 'v0')", bad)
                .execute(&storage.pool)
                .await
                .unwrap();
        }

        let found = storage
            .find_by_provider_id(&id, &provider, &ContentId::try_from("c1").unwrap())
            .await
            .unwrap();

        assert_eq!(found.map(|f| f.file_name).as_deref(), Some("good.jar"));
    }

    #[tokio::test]
    async fn should_drop_a_stored_path_that_changes_when_normalized() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        sqlx::query!(r"INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES ('inst', 'mods\a.jar', 'a.jar', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa')")
            .execute(&storage.pool)
            .await
            .unwrap();

        assert!(storage.get_pack(&id).await.unwrap().files.is_empty());
        assert!(storage.get_pack_files(&id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_change_only_the_hash_when_rehashing_a_record_with_an_invalid_one() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let path = ContentPath::parse("mods/a.jar").unwrap();
        sqlx::query!("INSERT INTO pack_files (instance_id, content_path, file_name, name, side, hash) VALUES ('inst', 'mods/a.jar', 'a.jar', 'Sodium', 'client', 'bad')")
            .execute(&storage.pool)
            .await
            .unwrap();
        sqlx::query!("INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES ('inst', 'mods/a.jar', 'malformed', 'c2', 'v2')")
            .execute(&storage.pool)
            .await
            .unwrap();

        let rehashed = storage
            .rehash_pack_file(&id, &path, &sha1_hash('c'))
            .await
            .unwrap()
            .expect("the record exists");

        assert_eq!(rehashed.hash, sha1_hash('c'));
        assert_eq!(rehashed.name.as_deref(), Some("Sodium"));
        assert_eq!(rehashed.side, Some(FileSide::Client));
        let kept = sqlx::query_scalar!("SELECT COUNT(*) FROM pack_file_updates WHERE content_path = 'mods/a.jar' AND provider_id = 'malformed'")
        .fetch_one(&storage.pool)
        .await
        .unwrap();
        assert_eq!(kept, 1);
    }

    #[tokio::test]
    async fn should_return_none_when_rehashing_a_file_with_no_row() {
        let storage = SqlitePackStorage::in_memory().await;

        let rehashed = storage
            .rehash_pack_file(
                &InstanceId::try_from("inst").unwrap(),
                &ContentPath::parse("mods/a.jar").unwrap(),
                &sha1_hash('c'),
            )
            .await
            .unwrap();

        assert!(rehashed.is_none());
    }

    #[tokio::test]
    async fn should_keep_the_stored_record_when_inserting_a_file_that_is_already_recorded() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let path = cp("mods/a.jar");
        let owner = PlanOrigin {
            provider_id: "plugin#cap".parse().unwrap(),
            content_id: ContentId::try_from("pack").unwrap(),
        };
        let planned = PackFile {
            name: Some("Sodium".into()),
            side: Some(FileSide::Client),
            installed_by: Some(owner.clone()),
            ..PackFile::from_hash("a.jar".into(), sha1_hash('a'))
        };
        storage
            .update_pack_file(&id, &path, &planned)
            .await
            .unwrap();

        let stored = storage
            .insert_pack_file_if_absent(
                &id,
                &path,
                &PackFile::from_hash("a.jar".into(), sha1_hash('b')),
            )
            .await
            .unwrap();

        for read in [stored, storage.get_pack_file(&id, &path).await.unwrap()] {
            assert_eq!(read.hash, sha1_hash('a'));
            assert_eq!(read.name.as_deref(), Some("Sodium"));
            assert_eq!(read.side, Some(FileSide::Client));
            assert_eq!(read.installed_by.unwrap().content_id, owner.content_id);
        }
    }

    #[tokio::test]
    async fn should_store_the_file_when_inserting_one_with_no_record() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let path = cp("mods/a.jar");
        let file = PackFile::from_hash("a.jar".into(), sha1_hash('a'));

        let stored = storage
            .insert_pack_file_if_absent(&id, &path, &file)
            .await
            .unwrap();

        assert_eq!(stored.hash, sha1_hash('a'));
        let read = storage.get_pack_file(&id, &path).await.unwrap();
        assert_eq!(read.hash, file.hash);
        assert_eq!(read.file_name, "a.jar");
        assert_eq!(storage.get_pack(&id).await.unwrap().files.len(), 1);
    }

    #[tokio::test]
    async fn should_repair_only_the_hash_when_inserting_a_file_whose_record_has_an_unusable_hash() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let path = cp("mods/a.jar");
        sqlx::query!("INSERT INTO packs (instance_id) VALUES ('inst')")
            .execute(&storage.pool)
            .await
            .unwrap();
        sqlx::query!("INSERT INTO pack_files (instance_id, content_path, file_name, name, hash, installed_by_provider_id, installed_by_content_id) VALUES ('inst', 'mods/a.jar', 'a.jar', 'Sodium', 'garbage', 'plugin#cap', 'pack')")
            .execute(&storage.pool)
            .await
            .unwrap();

        let stored = storage
            .insert_pack_file_if_absent(
                &id,
                &path,
                &PackFile::from_hash("a.jar".into(), sha1_hash('b')),
            )
            .await
            .unwrap();

        assert_eq!(stored.hash, sha1_hash('b'));
        assert_eq!(stored.name.as_deref(), Some("Sodium"));
        assert_eq!(stored.installed_by.unwrap().content_id.as_str(), "pack");
    }

    #[tokio::test]
    async fn should_keep_the_updates_of_a_stored_record_when_inserting_over_it() {
        let storage = SqlitePackStorage::in_memory().await;
        let id = InstanceId::try_from("inst").unwrap();
        let path = cp("mods/a.jar");
        let provider: ProviderId = "plugin#cap".parse().unwrap();
        let with_update = |content: &str| PackFile {
            update: Some(HashMap::from([(
                provider.clone(),
                ContentFileUpdateInfo {
                    content_id: ContentId::try_from(content).unwrap(),
                    version: None,
                },
            )])),
            ..PackFile::from_hash("a.jar".into(), sha1_hash('a'))
        };

        storage
            .insert_pack_file_if_absent(&id, &path, &with_update("first"))
            .await
            .unwrap();
        storage
            .insert_pack_file_if_absent(&id, &path, &with_update("second"))
            .await
            .unwrap();

        let read = storage.get_pack_file(&id, &path).await.unwrap();
        assert_eq!(read.update.unwrap()[&provider].content_id.as_str(), "first");
    }
}
