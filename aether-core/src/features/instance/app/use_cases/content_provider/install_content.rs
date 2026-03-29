use std::sync::Arc;

use crate::{
    features::instance::{
        ContentInstallParams, ContentProvider, ContentType, InstanceError, PackFile, PackStorage,
    },
    shared::CapabilityRegistry,
};

pub struct InstallContentUseCase<PS: PackStorage, CP: CapabilityRegistry<Arc<dyn ContentProvider>>>
{
    pack_storage: Arc<PS>,
    provider_registry: Arc<CP>,
}

impl<PS: PackStorage, CP: CapabilityRegistry<Arc<dyn ContentProvider>>>
    InstallContentUseCase<PS, CP>
{
    pub fn new(pack_storage: Arc<PS>, provider_registry: Arc<CP>) -> Self {
        Self {
            pack_storage,
            provider_registry,
        }
    }

    pub async fn execute(&self, install_params: ContentInstallParams) -> Result<(), InstanceError> {
        let provider_id = install_params.provider().to_owned();

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.to_string(),
                capability_id: provider_id.capability_id.to_string(),
            })?;

        match install_params {
            ContentInstallParams::Atomic(params) => {
                if params.content_type == ContentType::Modpack {
                    return Err(InstanceError::UnsupportedContentType {
                        content_type: params.content_type,
                    });
                }

                let instance_file = provider.capability.install_atomic(&params).await?;

                self.pack_storage
                    .update_pack_file(
                        &params.instance_id,
                        &instance_file.content_path.clone(),
                        &instance_file.into(),
                    )
                    .await?;
            }
            ContentInstallParams::Modpack(params) => {
                let (instance_id, processed_files) =
                    provider.capability.install_modpack(&params).await?;

                let content_files: Vec<PackFile> =
                    processed_files.iter().cloned().map(Into::into).collect();

                let paths: Vec<String> = processed_files
                    .iter()
                    .map(|f| f.content_path.to_owned())
                    .collect();

                self.pack_storage
                    .update_pack_file_many(&instance_id, &paths, &content_files)
                    .await?;
            }
        }

        Ok(())
    }
}
