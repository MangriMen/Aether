use async_trait::async_trait;
use extism_convert::Msgpack;

use crate::features::{
    instance::{Importer, ImporterCapabilityMetadata, InstanceError, InstanceId},
    plugins::{PluginError, PluginHandle, PluginImportInstance, PluginImporterCapability},
};

use crate::features::plugins::infra::extism::models::PluginInstanceExt;
use crate::features::plugins::infra::pack_instances::deploying_packs;
use crate::features::plugins::infra::plugin_utils::to_wasi_path;

/// `http://...` or `https://...`: the only addresses an importer is given instead of a path.
/// Any other `scheme://` is left to the path normalization, as is a Windows drive (`C://`).
fn is_url(input: &str) -> bool {
    input.split_once("://").is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    })
}

pub struct PluginImporterProxy {
    instance: PluginHandle,
    capability: PluginImporterCapability,
}

impl PluginImporterProxy {
    pub fn new(instance: PluginHandle, capability: PluginImporterCapability) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl Importer for PluginImporterProxy {
    fn metadata(&self) -> &ImporterCapabilityMetadata {
        &self.capability
    }

    async fn import(&self, path: &str) -> Result<Vec<InstanceId>, InstanceError> {
        let plugin_id = self.instance.id();
        let failed = |reason: String| InstanceError::ImportFailed {
            plugin_id: plugin_id.to_string(),
            capability_id: self.capability.id.to_string(),
            reason,
        };

        // Normalize Windows paths to WASI-compatible /mnt/<letter>/... format; a URL is not a
        // path and goes to the plugin as is (path normalization would turn `https://` into `https:/`)
        let params = PluginImportInstance {
            importer_id: self.capability.id.to_string(),
            path: if is_url(path) {
                path.to_owned()
            } else {
                to_wasi_path(path)
            },
        };
        let handler = self.capability.handler.clone();
        let deploying = plugin_id.to_owned();

        // The importer makes the instance and fills it in one call: its pack instances are
        // ready once the call is over.
        let result = self
            .instance
            .run(&self.capability.handler, move |plugin| {
                plugin.supports(&handler).then(|| {
                    let (result, instances) = deploying_packs(&deploying, || {
                        plugin.call::<_, ()>(&handler, Msgpack(params))
                    });
                    result.map(|()| instances)
                })
            })
            .await
            .and_then(Option::transpose);

        match result {
            Ok(Some(instances)) => Ok(instances),
            Err(PluginError::Stopped { plugin_id }) => Err(InstanceError::PluginNotRunning {
                plugin_id: plugin_id.into(),
            }),
            Err(err) => {
                tracing::error!("Error importing instance by plugin '{plugin_id}': {err:?}");
                Err(failed(err.to_string()))
            }
            Ok(None) => {
                tracing::error!(
                    "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                    plugin_id,
                    self.capability.handler,
                    self.capability.id
                );
                Err(failed(format!(
                    "the plugin has no function '{}'",
                    self.capability.handler
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use extism::{Function, Manifest, PluginBuilder, UserData, Wasm};

    use super::*;
    use crate::features::instance::{CapabilityId, PluginId};
    use crate::features::{
        instance::CapabilityMetadata,
        plugins::infra::{
            extism::models::ExtismPluginInstance,
            pack_instances::{is_deploying_packs, record_pack_instance},
        },
    };

    /// Exports `import`, which calls the host function `create` once, as an importer calls
    /// `instance_create`.
    const IMPORTER: &str = r#"(module
        (import "extism:host/user" "create" (func $create (result i64)))
        (func (export "import") (result i32)
            (drop (call $create))
            (i32.const 0)))"#;

    /// `create` does what `instance_create` does with an instance for a pack: records it when
    /// its plugin is deploying one. It runs inside the plugin's call, on the call's thread.
    fn importer() -> PluginImporterProxy {
        let create = Function::new(
            "create",
            [],
            [extism::ValType::I64],
            UserData::new(()),
            |_, _, outputs, _| {
                if is_deploying_packs(&PluginId::from_static("packwiz")) {
                    record_pack_instance(
                        &PluginId::from_static("packwiz"),
                        &InstanceId::try_from("pack").unwrap(),
                    );
                }
                outputs[0] = extism::Val::I64(0);
                Ok(())
            },
        );
        let plugin = PluginBuilder::new(Manifest::new([Wasm::data(IMPORTER.as_bytes().to_vec())]))
            .with_functions([create])
            .build()
            .expect("the test plugin should load");

        PluginImporterProxy::new(
            PluginHandle::new(ExtismPluginInstance::new(
                plugin,
                PluginId::from_static("packwiz"),
            )),
            PluginImporterCapability {
                metadata: ImporterCapabilityMetadata {
                    base: CapabilityMetadata {
                        id: CapabilityId::from_static("packwiz"),
                        name: "Packwiz".into(),
                        description: None,
                        icon: None,
                    },
                    field_label: None,
                    supported_extensions: Vec::new(),
                },
                handler: "import".into(),
            },
        )
    }

    #[test]
    fn should_tell_a_url_from_a_path_when_given_either() {
        assert!(is_url("https://example.com/pack.toml"));
        assert!(is_url("http://example.com/pack.toml"));
        assert!(is_url("HTTPS://example.com/pack.toml"));
        assert!(!is_url("ftp://example.com/pack.toml"));
        assert!(!is_url("file:///c/pack.toml"));
        assert!(!is_url(r"C:\packs\pack.toml"));
        assert!(!is_url("C://packs//pack.toml"));
        assert!(!is_url("/mnt/c/pack.toml"));
        assert!(!is_url("pack.toml"));
    }

    /// The instance the plugin creates in its host function comes back from the import: the
    /// deployment is seen across the plugin's call, on the blocking pool.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_give_back_the_pack_instances_when_they_were_created_during_the_import() {
        let instances = importer().import("pack.toml").await.unwrap();

        assert_eq!(instances, [InstanceId::try_from("pack").unwrap()]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_keep_the_reason_when_the_plugin_lacks_the_handler() {
        let mut proxy = importer();
        proxy.capability.handler = "missing".into();

        let err = proxy.import("pack.toml").await.unwrap_err();

        assert!(
            matches!(&err, InstanceError::ImportFailed { reason, .. } if reason.contains("missing")),
            "{err}"
        );
        assert!(err.to_string().contains("missing"), "{err}");
    }
}
