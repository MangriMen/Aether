use std::path::Path;

use tracing::info;

use crate::{
    features::auth::domain::Credential,
    features::auth::{
        app::CredentialsStorage,
        infra::{FsCredentialsStorage, SqliteCredentialsStorage},
    },
    shared::archive::infra::remove_legacy_file,
};

/// Moves legacy `credentials.json` into `SQLite`. The file holds plaintext tokens, so it is
/// deleted once the database is the source of truth, or when it cannot be read.
pub async fn migrate_credentials_to_sqlite(
    fs_storage: &FsCredentialsStorage,
    sqlite_storage: &SqliteCredentialsStorage,
    migrated_dir_name: &str,
) -> crate::Result<()> {
    let old_path = fs_storage.get_file_path();

    // Earlier versions archived the file (or set it aside as `.corrupt`); drop those copies.
    let migrated_path = old_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(migrated_dir_name);
    remove_legacy_file(&migrated_path.join("credentials.json"), "credentials").await;
    remove_legacy_file(
        &migrated_path.join("credentials.json.corrupt"),
        "credentials",
    )
    .await;

    // No need to migrate
    if !old_path.exists() {
        return Ok(());
    }

    if !sqlite_storage.list().await?.is_empty() {
        remove_legacy_file(&old_path, "credentials").await;
        return Ok(());
    }

    let old_credentials = match fs_storage.list().await {
        Ok(value) => value,
        Err(err) => {
            remove_legacy_file(&old_path, "credentials").await;
            return Err(err.into());
        }
    };

    if old_credentials.is_empty() {
        remove_legacy_file(&old_path, "credentials").await;
        return Ok(());
    }

    info!("Legacy credentials found. Starting migration to SQLite...");

    sqlite_storage
        .upsert_all(keep_first_active(old_credentials))
        .await?;

    remove_legacy_file(&old_path, "credentials").await;

    Ok(())
}

/// The database holds one active account, but an old file may list several.
fn keep_first_active(mut credentials: Vec<Credential>) -> Vec<Credential> {
    let mut seen_active = false;
    for credential in &mut credentials {
        if credential.is_active() {
            if seen_active {
                credential.deactivate();
            }
            seen_active = true;
        }
    }
    credentials
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;

    use crate::features::auth::tests::fixtures::CredentialsBuilder;

    use super::*;

    async fn sqlite() -> SqliteCredentialsStorage {
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
        SqliteCredentialsStorage::new(pool)
    }

    async fn legacy_file(dir: &Path, credentials: &[Credential]) -> FsCredentialsStorage {
        let fs = FsCredentialsStorage::new(dir);
        fs.upsert_all(credentials.to_vec()).await.unwrap();
        fs
    }

    #[test]
    fn should_keep_only_the_first_active_account_when_several_are_active() {
        let first = CredentialsBuilder::new("First").with_active(true).build();
        let second = CredentialsBuilder::new("Second").with_active(true).build();
        let third = CredentialsBuilder::new("Third").build();

        let result = keep_first_active(vec![first.clone(), second, third]);

        let active: Vec<_> = result
            .iter()
            .filter(|c| c.is_active())
            .map(Credential::id)
            .collect();
        assert_eq!(active, vec![first.id()]);
    }

    #[tokio::test]
    async fn should_delete_the_old_file_without_keeping_a_copy_after_migration() {
        let dir = tempfile::tempdir().unwrap();
        let account = CredentialsBuilder::new("Alice").with_active(true).build();
        let fs = legacy_file(dir.path(), std::slice::from_ref(&account)).await;
        let sqlite = sqlite().await;

        migrate_credentials_to_sqlite(&fs, &sqlite, "migrated")
            .await
            .unwrap();

        assert!(!fs.get_file_path().exists());
        assert!(
            !dir.path()
                .join("migrated")
                .join("credentials.json")
                .exists()
        );
        let stored = sqlite.list().await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id(), account.id());
    }

    #[tokio::test]
    async fn should_delete_leftovers_in_migrated_even_without_an_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let migrated = dir.path().join("migrated");
        std::fs::create_dir_all(&migrated).unwrap();
        std::fs::write(migrated.join("credentials.json"), "[]").unwrap();
        std::fs::write(migrated.join("credentials.json.corrupt"), "{").unwrap();
        let fs = FsCredentialsStorage::new(dir.path());

        migrate_credentials_to_sqlite(&fs, &sqlite().await, "migrated")
            .await
            .unwrap();

        assert!(!migrated.join("credentials.json").exists());
        assert!(!migrated.join("credentials.json.corrupt").exists());
    }

    #[tokio::test]
    async fn should_delete_the_old_file_and_keep_the_database_when_it_is_populated() {
        let dir = tempfile::tempdir().unwrap();
        let stored = CredentialsBuilder::new("Stored").with_active(true).build();
        let legacy = CredentialsBuilder::new("Legacy").build();
        let fs = legacy_file(dir.path(), &[legacy]).await;
        let sqlite = sqlite().await;
        sqlite.upsert(stored.clone()).await.unwrap();

        migrate_credentials_to_sqlite(&fs, &sqlite, "migrated")
            .await
            .unwrap();

        assert!(!fs.get_file_path().exists());
        let after = sqlite.list().await.unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].id(), stored.id());
    }

    #[tokio::test]
    async fn should_delete_an_empty_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsCredentialsStorage::new(dir.path());
        std::fs::write(fs.get_file_path(), "[]").unwrap();

        migrate_credentials_to_sqlite(&fs, &sqlite().await, "migrated")
            .await
            .unwrap();

        assert!(!fs.get_file_path().exists());
    }

    #[tokio::test]
    async fn should_delete_an_unreadable_file_and_return_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsCredentialsStorage::new(dir.path());
        std::fs::write(fs.get_file_path(), "{ not json").unwrap();

        let result = migrate_credentials_to_sqlite(&fs, &sqlite().await, "migrated").await;

        assert!(result.is_err());
        assert!(!fs.get_file_path().exists());
        assert!(
            !dir.path()
                .join("migrated")
                .join("credentials.json.corrupt")
                .exists()
        );
    }

    #[tokio::test]
    async fn should_not_fail_when_the_file_cannot_be_removed() {
        let dir = tempfile::tempdir().unwrap();
        // A directory in place of the file makes the removal fail with a non-NotFound error.
        let path = dir.path().join("credentials.json");
        std::fs::create_dir(&path).unwrap();

        remove_legacy_file(&path, "credentials").await;

        assert!(path.exists());
    }
}
