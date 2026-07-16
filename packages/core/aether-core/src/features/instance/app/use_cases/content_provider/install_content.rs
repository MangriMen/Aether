use async_trait::async_trait;
use path_slash::PathBufExt;
use std::sync::Arc;
use tracing::error;

use crate::features::instance::app::ports::InstallContentUseCasePort;
use crate::{
    features::instance::{
        AtomicInstallParams, ContentProvider, ContentType, InstanceError, PackStorage,
        app::ContentFileService,
    },
    shared::capability::domain::CapabilityRegistry,
};

pub struct InstallContentUseCase {
    pack_storage: Arc<dyn PackStorage>,
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    content_file_service: Arc<dyn ContentFileService>,
}

impl InstallContentUseCase {
    pub fn new(
        pack_storage: Arc<dyn PackStorage>,
        provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        content_file_service: Arc<dyn ContentFileService>,
    ) -> Self {
        Self {
            pack_storage,
            provider_registry,
            content_file_service,
        }
    }

    pub async fn execute(&self, params: AtomicInstallParams) -> Result<(), InstanceError> {
        if params.content_type == ContentType::Modpack {
            return Err(InstanceError::UnsupportedContentType {
                content_type: params.content_type,
            });
        }

        let provider_id = params.provider_id.clone();

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.clone(),
                capability_id: provider_id.capability_id.clone(),
            })?;

        let old_file = self
            .pack_storage
            .find_by_provider_id(&params.instance_id, &params.provider_id, &params.content_id)
            .await?;

        let instance_file = provider.capability.install_atomic(&params).await?;

        let install_result = async {
            if let Some(old_file) = old_file {
                self.pack_storage
                    .remove_pack_file(
                        &params.instance_id,
                        &params
                            .content_type
                            .get_relative_path(&old_file.file_name)
                            .to_slash_lossy(),
                    )
                    .await?;
            }

            let content_path = instance_file.metadata.content_path.clone();

            self.content_file_service
                .install_content_file(&params.instance_id, &content_path, &instance_file.temp_path)
                .await?;

            self.pack_storage
                .update_pack_file(
                    &params.instance_id,
                    &content_path,
                    &instance_file.metadata.into(),
                )
                .await?;

            Ok::<(), InstanceError>(())
        }
        .await;

        if install_result.is_err() {
            error!("Failed to install content: {:?}", install_result);
        }

        Ok(())
    }
}

#[async_trait]
impl InstallContentUseCasePort for InstallContentUseCase {
    async fn execute(&self, params: AtomicInstallParams) -> Result<(), InstanceError> {
        self.execute(params).await
    }
}
