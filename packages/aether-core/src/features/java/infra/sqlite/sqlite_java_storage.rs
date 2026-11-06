use async_trait::async_trait;
use sqlx::SqlitePool;
use tracing::{trace, warn};

use crate::features::java::{Java, JavaDomainError, JavaPath, JavaStorage};

pub struct SqliteJavaStorage {
    pool: SqlitePool,
}

impl SqliteJavaStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn row(&self, major_version: u32) -> Result<Option<SqlJava>, JavaDomainError> {
        let v = i64::from(major_version);
        Ok(sqlx::query_as!(
            SqlJava,
            "SELECT * FROM java_versions WHERE major_version = ?",
            v
        )
        .fetch_optional(&self.pool)
        .await?)
    }
}

#[async_trait]
impl JavaStorage for SqliteJavaStorage {
    async fn list(&self) -> Result<Vec<Java>, JavaDomainError> {
        let rows = sqlx::query_as!(SqlJava, "SELECT * FROM java_versions")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(SqlJava::into_java).collect())
    }

    async fn get(&self, major_version: u32) -> Result<Option<Java>, JavaDomainError> {
        Ok(self.row(major_version).await?.and_then(SqlJava::into_java))
    }

    async fn upsert(&self, java: Java) -> Result<Java, JavaDomainError> {
        trace!(java=?java, "Upsert java");

        // A row `get` skips is still the user's entry: say so when it is replaced.
        if let Some(old) = self.row(java.major_version()).await?
            && JavaPath::try_from(old.path.clone()).is_err()
        {
            warn!(
                major_version = old.major_version,
                old_path = %old.path,
                new_path = %java.path(),
                "Replacing a stored Java whose path is invalid"
            );
        }

        let sql = SqlJava::from(java.clone());

        sqlx::query!(
            "INSERT INTO java_versions (major_version, version, architecture, path)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(major_version) DO UPDATE SET
                version = excluded.version,
                architecture = excluded.architecture,
                path = excluded.path",
            sql.major_version,
            sql.version,
            sql.architecture,
            sql.path
        )
        .execute(&self.pool)
        .await?;

        Ok(java)
    }

    async fn remove(&self, major_version: u32) -> Result<(), JavaDomainError> {
        trace!(major_version=%major_version, "Remove java");

        let rows_affected = sqlx::query!(
            "DELETE FROM java_versions WHERE major_version = ?",
            major_version,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(JavaDomainError::NotFound {
                version: major_version,
            });
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SqlJava {
    major_version: i64,
    version: String,
    architecture: String,
    path: String,
}

impl From<Java> for SqlJava {
    fn from(j: Java) -> Self {
        Self {
            major_version: i64::from(j.major_version()),
            version: j.version().to_string(),
            architecture: j.architecture().to_string(),
            path: j.path().to_string(),
        }
    }
}

impl SqlJava {
    /// The stored Java; a row whose path is not a valid [`JavaPath`] (one saved before paths were
    /// checked) is skipped with a warning instead of failing every read of the table. Such a
    /// runtime is treated as not installed, so the version gets installed again when needed.
    fn into_java(self) -> Option<Java> {
        let path = match JavaPath::try_from(self.path) {
            Ok(path) => path,
            Err(err) => {
                warn!(
                    major_version = self.major_version,
                    error = %err,
                    "Skipping a stored Java with an invalid path"
                );
                return None;
            }
        };

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Some(Java::new(
            self.major_version as u32,
            self.version,
            self.architecture,
            path,
        ))
    }
}

impl From<sqlx::Error> for JavaDomainError {
    fn from(err: sqlx::Error) -> Self {
        JavaDomainError::Storage(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;

    use super::*;

    async fn storage() -> SqliteJavaStorage {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        crate::core::migrations::run_migrations(&pool)
            .await
            .unwrap();
        SqliteJavaStorage::new(pool)
    }

    #[tokio::test]
    async fn should_skip_only_the_invalid_row_when_a_stored_path_is_relative() {
        let storage = storage().await;
        let valid_path = JavaPath::try_from(std::env::temp_dir().join("javaw").as_path()).unwrap();
        storage
            .upsert(Java::new(
                17,
                "17.0.1".into(),
                "x64".into(),
                valid_path.clone(),
            ))
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO java_versions (major_version, version, architecture, path)
             VALUES (8, 'Custom', 'Unknown', 'java')",
        )
        .execute(&storage.pool)
        .await
        .unwrap();

        let listed = storage.list().await.unwrap();

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].path(), &valid_path);
        assert_eq!(storage.get(8).await.unwrap(), None);
    }
}
