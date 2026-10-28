use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::features::auth::{
    app::{AuthApplicationError, CredentialsStorage},
    domain::{AuthDomainError, Credential},
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

    async fn upsert_all(
        &self,
        mut credentials: Vec<Credential>,
    ) -> Result<(), AuthApplicationError> {
        // The inactive ones go first: the unique index allows one active row, so the previous one
        // must be written inactive before the new one is written active.
        credentials.sort_by_key(Credential::is_active);

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

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;

    use crate::features::auth::{app::ActiveAccountHelper, tests::fixtures::CredentialsBuilder};

    use super::*;

    const ONE_ACTIVE: &str =
        include_str!("../../../../../migrations/20261029120000_credentials_one_active.sql");

    async fn pool() -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        sqlx::pool::PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap()
    }

    async fn storage() -> SqliteCredentialsStorage {
        let pool = pool().await;
        crate::core::migrations::run_migrations(&pool)
            .await
            .unwrap();
        SqliteCredentialsStorage::new(pool)
    }

    #[tokio::test]
    async fn should_move_the_active_account_to_the_target_when_it_is_set_active() {
        let storage = storage().await;
        let first = CredentialsBuilder::new("First").with_active(true).build();
        let second = CredentialsBuilder::new("Second").build();
        storage.upsert(first.clone()).await.unwrap();
        storage.upsert(second.clone()).await.unwrap();

        ActiveAccountHelper::set_active(&storage, second.id())
            .await
            .unwrap();

        assert_eq!(
            storage.find_active().await.unwrap().unwrap().id(),
            second.id()
        );
        assert!(!storage.get(first.id()).await.unwrap().is_active());
    }

    #[tokio::test]
    async fn should_refuse_a_second_active_account_when_one_is_already_active() {
        let storage = storage().await;
        storage
            .upsert(CredentialsBuilder::new("First").with_active(true).build())
            .await
            .unwrap();

        let result = storage
            .upsert(CredentialsBuilder::new("Second").with_active(true).build())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_keep_the_first_active_account_when_the_index_is_added_over_several() {
        let pool = pool().await;
        sqlx::raw_sql(
            "CREATE TABLE credentials (
               id TEXT PRIMARY KEY NOT NULL, username TEXT NOT NULL, account_type TEXT NOT NULL,
               is_active BOOLEAN NOT NULL DEFAULT 0, access_token TEXT NOT NULL,
               refresh_token TEXT NOT NULL, expires TEXT NOT NULL);
             INSERT INTO credentials VALUES
               ('a', 'A', 'offline', 1, 'null', 'null', '2099-01-01T00:00:00+00:00'),
               ('b', 'B', 'offline', 1, 'null', 'null', '2099-01-01T00:00:00+00:00'),
               ('c', 'C', 'offline', 0, 'null', 'null', '2099-01-01T00:00:00+00:00')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::raw_sql(ONE_ACTIVE).execute(&pool).await.unwrap();

        let active: Vec<String> =
            sqlx::query_scalar("SELECT id FROM credentials WHERE is_active = 1")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(active, vec!["a".to_owned()]);
    }
}
