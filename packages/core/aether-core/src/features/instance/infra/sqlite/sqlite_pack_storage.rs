use crate::features::instance::{
    ContentFileUpdateInfo, InstanceError, Pack, PackEntry, PackFile, PackStorage, ProviderId,
};
use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;
use std::str::FromStr;

pub struct SqlitePackStorage {
    pool: SqlitePool,
}

impl SqlitePackStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // Helper to internalize update logic for reuse in transactions
    async fn upsert_file_internal(
        tx: &mut Transaction<'_, Sqlite>,
        instance_id: &str,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        // Ensure pack exists
        sqlx::query!(
            "INSERT OR IGNORE INTO packs (instance_id) VALUES (?)",
            instance_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let update_provider_id = pack_file
            .update_provider_id
            .as_ref()
            .map(std::string::ToString::to_string);

        sqlx::query!(
            r#"
            INSERT INTO pack_files (instance_id, content_path, file_name, name, hash, side, update_provider_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(instance_id, content_path) DO UPDATE SET 
                file_name = excluded.file_name, 
                name = excluded.name, 
                hash = excluded.hash, 
                side = excluded.side, 
                update_provider_id = excluded.update_provider_id
            "#,
            instance_id,
            content_path,
            pack_file.file_name,
            pack_file.name,
            pack_file.hash,
            pack_file.side,
            update_provider_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        // Cleanup old updates
        sqlx::query!(
            "DELETE FROM pack_file_updates WHERE instance_id = ? AND content_path = ?",
            instance_id,
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
                    instance_id,
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
    async fn get_pack(&self, instance_id: &str) -> Result<Pack, InstanceError> {
        let files = sqlx::query!(
            "SELECT content_path FROM pack_files WHERE instance_id = ? ORDER BY content_path ASC",
            instance_id
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

    async fn update_pack(&self, _instance_id: &str, _pack: &Pack) -> Result<(), InstanceError> {
        // In SQL, the "pack" is just a collection of files.
        // Individual file updates manage this state.
        Ok(())
    }

    async fn get_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<PackFile, InstanceError> {
        let row = sqlx::query!(
            "SELECT file_name, name, hash, side, update_provider_id FROM pack_files WHERE instance_id = ? AND content_path = ?",
            instance_id,
            content_path
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?
        .ok_or_else(|| InstanceError::Storage(format!("Pack file not found: {content_path}")))?;

        let updates = sqlx::query!(
            "SELECT provider_id, content_id, version_id FROM pack_file_updates WHERE instance_id = ? AND content_path = ?",
            instance_id,
            content_path
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;

        let mut update_map = HashMap::new();
        for u in updates {
            update_map.insert(
                ProviderId::from_str(&u.provider_id).map_err(InstanceError::Storage)?,
                ContentFileUpdateInfo {
                    content_id: u.content_id,
                    version: u.version_id,
                },
            );
        }

        Ok(PackFile {
            file_name: row.file_name,
            name: row.name,
            hash: row.hash,
            download: None, // TODO: Add to schema if needed
            option: None,   // TODO: Add to schema if needed
            side: row.side,
            update_provider_id: row
                .update_provider_id
                .and_then(|p| ProviderId::from_str(&p).ok()),
            update: if update_map.is_empty() {
                None
            } else {
                Some(update_map)
            },
        })
    }

    async fn find_by_provider_id(
        &self,
        instance_id: &str,
        provider_id: &ProviderId,
        content_id: &str,
    ) -> Result<Option<PackFile>, InstanceError> {
        let pid = provider_id.to_string();
        let row = sqlx::query!(
            "SELECT content_path FROM pack_file_updates WHERE instance_id = ? AND provider_id = ? AND content_id = ? LIMIT 1",
            instance_id,
            pid,
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
        instance_id: &str,
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
        instance_id: &str,
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
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), InstanceError> {
        sqlx::query!(
            "DELETE FROM pack_files WHERE instance_id = ? AND content_path = ?",
            instance_id,
            content_path
        )
        .execute(&self.pool)
        .await
        .map_err(|e| InstanceError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn remove_pack_file_many(
        &self,
        instance_id: &str,
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
                instance_id,
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
}
