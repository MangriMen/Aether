use std::sync::Arc;

use crate::features::update::UpdateService;

pub struct InstallUpdateUseCase<US: UpdateService> {
    update_service: Arc<US>,
}

impl<US: UpdateService> InstallUpdateUseCase<US> {
    pub fn new(update_service: Arc<US>) -> Self {
        Self { update_service }
    }

    pub async fn execute(&self) -> Result<(), String> {
        self.update_service.install().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::update::UpdateStatus;
    use async_trait::async_trait;

    struct MockUpdateService {
        install_result: Result<(), String>,
    }

    #[async_trait]
    impl UpdateService for MockUpdateService {
        async fn check(&self) -> Result<Option<UpdateStatus>, String> {
            Ok(None)
        }

        async fn install(&self) -> Result<(), String> {
            self.install_result.clone()
        }
    }

    #[tokio::test]
    async fn install_success() {
        let service = Arc::new(MockUpdateService {
            install_result: Ok(()),
        });
        let use_case = InstallUpdateUseCase::new(service);
        assert!(use_case.execute().await.is_ok());
    }

    #[tokio::test]
    async fn install_failure() {
        let service = Arc::new(MockUpdateService {
            install_result: Err("download failed".to_string()),
        });
        let use_case = InstallUpdateUseCase::new(service);
        let result = use_case.execute().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "download failed");
    }
}
