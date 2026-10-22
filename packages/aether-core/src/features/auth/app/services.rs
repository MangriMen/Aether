use crate::features::auth::{
    app::{AuthApplicationError, CredentialsStorage},
    domain::{AuthDomainError, Credential},
};
use uuid::Uuid;

pub struct ActiveAccountHelper;

impl ActiveAccountHelper {
    /// Returns the active credential, promoting the first one that can be
    /// activated when none is active. Accounts that cannot be activated (for
    /// example an expired Microsoft token) are skipped instead of failing the
    /// whole operation, so a single stale account never blocks the others.
    pub async fn ensure_active(
        storage: &dyn CredentialsStorage,
    ) -> Result<Credential, AuthApplicationError> {
        let mut all = storage.list().await?;

        if let Some(active) = all.iter().find(|c| c.is_active()) {
            return Ok(active.clone());
        }

        let mut activation_error = None;
        let mut promoted = None;

        for credentials in &mut all {
            match credentials.activate() {
                Ok(()) => {
                    promoted = Some(credentials.clone());
                    break;
                }
                Err(error) => {
                    activation_error.get_or_insert(error);
                }
            }
        }

        let promoted = promoted.ok_or(AuthApplicationError::Domain(
            activation_error.unwrap_or(AuthDomainError::NoActiveCredentials),
        ))?;

        // Every other credential is already inactive, so only the promoted one
        // needs to be written back.
        storage.upsert(promoted.clone()).await?;

        Ok(promoted)
    }

    pub async fn set_active(
        storage: &dyn CredentialsStorage,
        id: Uuid,
    ) -> Result<Credential, AuthApplicationError> {
        let mut all = storage.list().await?;

        let mut target_idx = None;
        for (i, credentials) in all.iter_mut().enumerate() {
            if credentials.id() == id {
                credentials.activate()?;
                target_idx = Some(i);
            } else {
                credentials.deactivate();
            }
        }

        let idx = target_idx.ok_or(AuthApplicationError::Domain(
            AuthDomainError::CredentialsNotFound { id },
        ))?;

        let result = all[idx].clone();
        storage.upsert_all(all).await?;

        Ok(result)
    }
}
