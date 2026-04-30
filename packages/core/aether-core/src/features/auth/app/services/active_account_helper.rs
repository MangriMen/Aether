use crate::features::auth::{
    AuthApplicationError, AuthDomainError, Credential, CredentialsStorage,
};
use uuid::Uuid;

pub struct ActiveAccountHelper;

impl ActiveAccountHelper {
    pub async fn ensure_active(
        storage: &dyn CredentialsStorage,
    ) -> Result<Credential, AuthApplicationError> {
        let all = storage.list().await?;

        if let Some(active) = all.iter().find(|c| c.is_active()) {
            return Ok(active.clone());
        }

        let first = all.first().ok_or(AuthApplicationError::Domain(
            AuthDomainError::NoActiveCredentials,
        ))?;

        Self::set_active(storage, first.id()).await
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
