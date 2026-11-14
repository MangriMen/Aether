use std::path::Path;

use tracing::{info, warn};

use crate::{
    features::auth::domain::Credential,
    features::auth::{
        app::{AuthApplicationError, CredentialsStorage},
        infra::{FsCredentialsStorage, SqliteCredentialsStorage},
    },
    shared::{archive::infra::remove_legacy_file, io::domain::IoError},
};

/// Moves legacy `credentials.json` into `SQLite`. The file holds plaintext tokens, so it is
/// deleted once the database is the source of truth, or when it cannot be read.
pub async fn migrate_credentials_to_sqlite(
    fs_storage: &FsCredentialsStorage,
    sqlite_storage: &SqliteCredentialsStorage,
    migrated_dir_name: &str,
) -> crate::Result<()> {
    let old_path = fs_storage.get_file_path();
    let mut all_removed = true;

    // Earlier versions archived the file (or set it aside as `.corrupt`); drop those copies.
    let migrated_path = old_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(migrated_dir_name);
    all_removed &= remove_legacy_file(&migrated_path.join("credentials.json"), "credentials").await;
    all_removed &= remove_legacy_file(
        &migrated_path.join("credentials.json.corrupt"),
        "credentials",
    )
    .await;

    // No need to migrate
    if !old_path.exists() {
        return finish(all_removed, &migrated_path);
    }

    if !sqlite_storage.list().await?.is_empty() {
        all_removed &= remove_legacy_file(&old_path, "credentials").await;
        return finish(all_removed, &old_path);
    }

    let old_credentials = match fs_storage.read_legacy().await {
        Ok(value) => value,
        Err(err @ IoError::DeserializationError(_)) => {
            // Unparsable: it would fail the same way on every start, and it holds plaintext tokens.
            let removed = remove_legacy_file(&old_path, "credentials").await;
            if !removed {
                warn!("Unreadable legacy credentials file could not be removed: {old_path:?}");
            }
            return Err(AuthApplicationError::from(err).into());
        }
        // Busy, permission, timeout...: may succeed next start, so keep the file.
        Err(err) => return Err(AuthApplicationError::from(err).into()),
    };

    if old_credentials.is_empty() {
        all_removed &= remove_legacy_file(&old_path, "credentials").await;
        return finish(all_removed, &old_path);
    }

    info!("Legacy credentials found. Starting migration to SQLite...");

    sqlite_storage
        .upsert_all(keep_first_active(old_credentials))
        .await?;

    all_removed &= remove_legacy_file(&old_path, "credentials").await;

    finish(all_removed, &old_path)
}

/// A plaintext file that could not be deleted must reach the user, not only the log.
fn finish(all_removed: bool, path: &Path) -> crate::Result<()> {
    if all_removed {
        return Ok(());
    }
    Err(crate::ErrorKind::CoreError(format!(
        "Could not remove the legacy credentials file near {}",
        path.display()
    ))
    .as_error())
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

        let removed = remove_legacy_file(&path, "credentials").await;

        assert!(!removed);
        assert!(path.exists());
        assert!(remove_legacy_file(&dir.path().join("missing.json"), "credentials").await);
    }

    #[tokio::test]
    async fn should_return_an_error_when_a_leftover_in_migrated_cannot_be_removed() {
        let dir = tempfile::tempdir().unwrap();
        let account = CredentialsBuilder::new("Alice").with_active(true).build();
        let fs = legacy_file(dir.path(), std::slice::from_ref(&account)).await;
        let leftover = dir.path().join("migrated").join("credentials.json");
        std::fs::create_dir_all(&leftover).unwrap();
        let sqlite = sqlite().await;

        let result = migrate_credentials_to_sqlite(&fs, &sqlite, "migrated").await;

        assert!(result.is_err());
        // The other steps still ran.
        assert!(!fs.get_file_path().exists());
        assert_eq!(sqlite.list().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn should_keep_the_file_when_it_cannot_be_read_for_a_reason_other_than_parsing() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsCredentialsStorage::new(dir.path());
        // A directory in place of the file: exists, but reading it is an IO error, not a parse one.
        std::fs::create_dir(fs.get_file_path()).unwrap();

        let result = migrate_credentials_to_sqlite(&fs, &sqlite().await, "migrated").await;

        assert!(result.is_err());
        assert!(fs.get_file_path().exists());
    }
}
