use std::sync::Arc;

use crate::features::update::{UpdateError, UpdateService};

pub struct InstallUpdateUseCase<US: UpdateService> {
    update_service: Arc<US>,
}

impl<US: UpdateService> InstallUpdateUseCase<US> {
    pub fn new(update_service: Arc<US>) -> Self {
        Self { update_service }
    }

    pub async fn execute(&self) -> Result<(), UpdateError> {
        self.update_service.install().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::update::UpdateStatus;
    use async_trait::async_trait;

    struct MockUpdateService {
        install_result: Result<(), UpdateError>,
    }

    #[async_trait]
    impl UpdateService for MockUpdateService {
        async fn check(&self) -> Result<Option<UpdateStatus>, UpdateError> {
            Ok(None)
        }

        async fn install(&self) -> Result<(), UpdateError> {
            self.install_result.clone()
        }
    }

    #[tokio::test]
    async fn install_failure() {
        let service = Arc::new(MockUpdateService {
            install_result: Err(UpdateError::NoPendingUpdate),
        });
        let use_case = InstallUpdateUseCase::new(service);
        let result = use_case.execute().await;
        assert!(matches!(result, Err(UpdateError::NoPendingUpdate)));
    }
}
