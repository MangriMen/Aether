use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::features::auth::{
    AuthApplicationError, AuthDomainError, Credential, CredentialsStorage,
};

pub struct SqliteCredentialsStorage {
    pool: SqlitePool,
}

impl SqliteCredentialsStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CredentialsStorage for SqliteCredentialsStorage {
    async fn list(&self) -> Result<Vec<Credential>, AuthApplicationError> {
        let rows = sqlx::query_as!(SqlCredential, "SELECT * FROM credentials")
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(Credential::try_from).collect()
    }

    async fn get(&self, id: Uuid) -> Result<Credential, AuthApplicationError> {
        let id_str = id.to_string();
        let row = sqlx::query_as!(
            SqlCredential,
            "SELECT * FROM credentials WHERE id = ?",
            id_str
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Credential::try_from(row),
            None => Err(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            )),
        }
    }

    async fn upsert(&self, credential: Credential) -> Result<Credential, AuthApplicationError> {
        let sql = SqlCredential::from(credential.clone());

        sqlx::query!(
            "INSERT INTO credentials (id, username, account_type, is_active, access_token, refresh_token, expires)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                username = excluded.username,
                account_type = excluded.account_type,
                is_active = excluded.is_active,
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                expires = excluded.expires",
            sql.id, sql.username, sql.account_type, sql.is_active,
            sql.access_token, sql.refresh_token, sql.expires
        )
        .execute(&self.pool)
        .await?;

        Ok(credential)
    }

    async fn upsert_all(&self, credentials: Vec<Credential>) -> Result<(), AuthApplicationError> {
        let mut tx = self.pool.begin().await?;

        for credential in credentials {
            let sql = SqlCredential::from(credential);

            sqlx::query!(
                "INSERT INTO credentials (id, username, account_type, is_active, access_token, refresh_token, expires)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                    is_active = excluded.is_active,
                    access_token = excluded.access_token,
                    refresh_token = excluded.refresh_token,
                    expires = excluded.expires",
                sql.id, sql.username, sql.account_type, sql.is_active,
                sql.access_token, sql.refresh_token, sql.expires
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError> {
        let id_str = id.to_string();
        let result = sqlx::query!("DELETE FROM credentials WHERE id = ?", id_str)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ));
        }
        Ok(())
    }

    async fn clear(&self) -> Result<(), AuthApplicationError> {
        sqlx::query!("DELETE FROM credentials")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_active(&self) -> Result<Option<Credential>, AuthApplicationError> {
        let row = sqlx::query_as!(
            SqlCredential,
            "SELECT * FROM credentials WHERE is_active = 1 LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(Credential::try_from).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct SqlCredential {
    id: String,
    username: String,
    account_type: String,
    is_active: bool,
    access_token: String,
    refresh_token: String,
    expires: String,
}

impl From<Credential> for SqlCredential {
    fn from(c: Credential) -> Self {
        Self {
            id: c.id().to_string(),
            username: c.username().to_string(),
            account_type: c.account_type().to_string(),
            is_active: c.is_active(),
            access_token: c.access_token().to_string(),
            refresh_token: c.refresh_token().to_string(),
            expires: c.expires().to_rfc3339(),
        }
    }
}

impl TryFrom<SqlCredential> for Credential {
    type Error = AuthApplicationError;

    fn try_from(value: SqlCredential) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&value.id)
            .map_err(|_| AuthApplicationError::Storage("Invalid UUID in DB".into()))?;

        let username = value.username.try_into()?;
        let account_type = value.account_type.try_into()?;

        let expires = DateTime::parse_from_rfc3339(&value.expires)
            .map_err(|_| AuthApplicationError::Storage("Invalid Date in DB".into()))?
            .with_timezone(&Utc);

        Ok(Self::new(
            id,
            username,
            account_type,
            value.is_active,
            value.access_token,
            value.refresh_token,
            expires,
        ))
    }
}

impl From<sqlx::Error> for AuthApplicationError {
    fn from(err: sqlx::Error) -> Self {
        AuthApplicationError::Storage(err.to_string())
    }
}
