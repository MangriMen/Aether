use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::features::java::{Java, JavaStorage, JavaStorageError};

pub struct SqliteJavaStorage {
    pool: SqlitePool,
}

impl SqliteJavaStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JavaStorage for SqliteJavaStorage {
    async fn list(&self) -> Result<Vec<Java>, JavaStorageError> {
        let rows = sqlx::query_as!(SqlJava, "SELECT * FROM java_versions")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Java::from).collect())
    }

    async fn get(&self, major_version: u32) -> Result<Option<Java>, JavaStorageError> {
        let v = i64::from(major_version);
        let row = sqlx::query_as!(
            SqlJava,
            "SELECT * FROM java_versions WHERE major_version = ?",
            v
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Java::from))
    }

    async fn upsert(&self, java: Java) -> Result<Java, JavaStorageError> {
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

impl From<SqlJava> for Java {
    fn from(value: SqlJava) -> Self {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Self::new(
            value.major_version as u32,
            value.version,
            value.architecture,
            value.path,
        )
    }
}

impl From<sqlx::Error> for JavaStorageError {
    fn from(err: sqlx::Error) -> Self {
        JavaStorageError::Storage(err.to_string())
    }
}
