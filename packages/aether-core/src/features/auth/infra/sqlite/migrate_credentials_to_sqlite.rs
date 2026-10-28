use tracing::info;

use crate::{
    features::auth::domain::Credential,
    features::auth::{
        app::CredentialsStorage,
        infra::{FsCredentialsStorage, SqliteCredentialsStorage},
    },
    shared::archive::infra::{archive_legacy_file, quarantine_unreadable_legacy_file},
};

pub async fn migrate_credentials_to_sqlite(
    fs_storage: &FsCredentialsStorage,
    sqlite_storage: &SqliteCredentialsStorage,
    migrated_dir_name: &str,
) -> crate::Result<()> {
    let old_path = fs_storage.get_file_path();

    // No need to migrate
    if !old_path.exists() {
        return Ok(());
    }

    if !sqlite_storage.list().await?.is_empty() {
        return Ok(());
    }

    let old_credentials = match fs_storage.list().await {
        Ok(value) => value,
        Err(err) => {
            quarantine_unreadable_legacy_file(&old_path, migrated_dir_name, "credentials").await;
            return Err(err.into());
        }
    };

    if old_credentials.is_empty() {
        return Ok(());
    }

    info!("Legacy credentials found. Starting migration to SQLite...");

    sqlite_storage
        .upsert_all(keep_first_active(old_credentials))
        .await?;

    archive_legacy_file(&old_path, migrated_dir_name, "credentials").await;

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
    use crate::features::auth::tests::fixtures::CredentialsBuilder;

    use super::*;

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
}
