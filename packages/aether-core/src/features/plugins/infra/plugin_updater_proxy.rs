use crate::features::plugins::infra::extism::Wire;
use aether_core_plugin_api::v0::UpdateInstanceParamsDto;
use async_trait::async_trait;

use crate::features::{
    instance::{InstanceError, InstanceId, Updater, UpdaterCapabilityMetadata},
    plugins::{PluginError, PluginHandle, PluginUpdaterCapability},
};

use crate::features::plugins::infra::extism::models::PluginInstanceExt;
use crate::features::plugins::infra::refusal::refusal;

pub struct PluginUpdaterProxy {
    instance: PluginHandle,
    capability: PluginUpdaterCapability,
}

impl PluginUpdaterProxy {
    pub fn new(instance: PluginHandle, capability: PluginUpdaterCapability) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl Updater for PluginUpdaterProxy {
    fn metadata(&self) -> &UpdaterCapabilityMetadata {
        &self.capability
    }

    async fn update(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let plugin_id = self.instance.id();
        let failed = || InstanceError::UpdateFailed {
            modpack_id: self.capability.id.to_string(),
        };

        let handler = self.capability.handler.clone();
        let instance_id = instance_id.clone();

        let result = self
            .instance
            .run(&self.capability.handler, move |plugin| {
                plugin.supports(&handler).then(|| {
                    plugin.call::<_, ()>(
                        &handler,
                        Wire(UpdateInstanceParamsDto {
                            instance_id: instance_id.as_str().to_owned(),
                        }),
                    )
                })
            })
            .await
            .and_then(Option::transpose);

        match result {
            Ok(Some(())) => Ok(()),
            Err(PluginError::Stopped { plugin_id }) => Err(InstanceError::PluginNotRunning {
                plugin_id: plugin_id.into(),
            }),
            Err(err) => {
                tracing::error!("Error updating instance by plugin '{plugin_id}': {err:?}");
                Err(refusal(&err).unwrap_or_else(failed))
            }
            Ok(None) => {
                tracing::error!(
                    "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                    plugin_id,
                    self.capability.handler,
                    self.capability.id
                );
                Err(failed())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::{CapabilityId, CapabilityMetadata, PluginId};
    use crate::features::plugins::CommandApprovalRefusal;
    use crate::features::plugins::infra::refusal::failing_plugin::{
        COMMAND_DENIED, HOST_REFUSED, NOT_FOUND, failing,
    };

    fn failing_updater(message: &str) -> PluginUpdaterProxy {
        PluginUpdaterProxy::new(
            failing(&PluginId::from_static("packwiz"), &["update"], message),
            PluginUpdaterCapability {
                metadata: UpdaterCapabilityMetadata {
                    base: CapabilityMetadata {
                        id: CapabilityId::from_static("packwiz"),
                        name: "Packwiz".into(),
                        description: None,
                        icon: None,
                    },
                },
                handler: "update".into(),
            },
        )
    }

    fn instance() -> InstanceId {
        InstanceId::try_from("pack").unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_name_the_refused_host_when_the_updater_reports_one() {
        let err = failing_updater(HOST_REFUSED)
            .update(&instance())
            .await
            .unwrap_err();

        assert!(
            matches!(&err, InstanceError::PluginHostNotAllowed { plugin_id, host, details }
                if plugin_id == "packwiz" && host == "cdn.example" && details == "cdn refused"),
            "{err:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_tell_why_the_command_did_not_run_when_the_updater_reports_a_refused_one() {
        let err = failing_updater(COMMAND_DENIED)
            .update(&instance())
            .await
            .unwrap_err();

        assert!(
            matches!(&err, InstanceError::PluginCommandNotApproved { plugin_id, reason, details }
                if plugin_id == "packwiz"
                    && *reason == CommandApprovalRefusal::Denied
                    && details == "java refused"),
            "{err:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_report_a_failed_update_when_the_updater_fails_another_way() {
        let err = failing_updater(NOT_FOUND)
            .update(&instance())
            .await
            .unwrap_err();

        assert!(
            matches!(&err, InstanceError::UpdateFailed { modpack_id } if modpack_id == "packwiz"),
            "{err:?}"
        );
    }
}
