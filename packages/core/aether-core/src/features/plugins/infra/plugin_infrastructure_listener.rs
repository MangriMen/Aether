use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    features::{
        events::{EventEmitter, PluginEvent, PluginEventType},
        instance::{ContentProvider, Importer, Updater},
        plugins::{
            infra::{PluginContentProviderProxy, PluginImporterProxy, PluginUpdaterProxy},
            AsCapabilityMetadata, PluginCapabilities, PluginError, PluginInstance, PluginRegistry,
            PluginState,
        },
    },
    shared::{CapabilityRegistry, IoError},
};

pub struct PluginInfrastructureListener<
    E: EventEmitter,
    IR: CapabilityRegistry<Arc<dyn Importer>>,
    UR: CapabilityRegistry<Arc<dyn Updater>>,
    CR: CapabilityRegistry<Arc<dyn ContentProvider>>,
> {
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<IR>,
    updaters_registry: Arc<UR>,
    content_providers_registry: Arc<CR>,
}

impl<E, IR, UR, CR> PluginInfrastructureListener<E, IR, UR, CR>
where
    E: EventEmitter,
    IR: CapabilityRegistry<Arc<dyn Importer>>,
    UR: CapabilityRegistry<Arc<dyn Updater>>,
    CR: CapabilityRegistry<Arc<dyn ContentProvider>>,
{
    pub fn new(
        plugin_registry: Arc<PluginRegistry<E>>,
        importers_registry: Arc<IR>,
        updaters_registry: Arc<UR>,
        content_providers_registry: Arc<CR>,
    ) -> Self {
        Self {
            plugin_registry,
            importers_registry,
            updaters_registry,
            content_providers_registry,
        }
    }

    pub async fn on_plugin_event(&self, data: String) {
        let result: Result<(), PluginError> = async {
            let plugin_event = serde_json::from_str::<PluginEvent>(&data)
                .map_err(|e| IoError::DeserializationError(e.to_string()))?;

            let plugin_id = match plugin_event.event {
                PluginEventType::Edit { plugin_id } => plugin_id,
                _ => return Ok(()),
            };

            let state = {
                let plugin = self.plugin_registry.get(&plugin_id)?;
                plugin.state.clone()
            };

            if matches!(state, PluginState::Loading) {
                return Ok(());
            }

            let capabilities = self.plugin_registry.get_capabilities(&plugin_id)?;

            if let Some(caps) = capabilities {
                match state {
                    PluginState::Loaded(instance) => {
                        self.sync_all_capabilities(&plugin_id, Some(instance), &caps)
                            .await?;
                    }
                    PluginState::NotLoaded | PluginState::Unloading | PluginState::Failed(_) => {
                        self.sync_all_capabilities(&plugin_id, None, &caps).await?;
                    }
                    _ => {}
                }
            }

            Ok(())
        }
        .await;

        if let Err(err) = result {
            tracing::error!("Error handling plugin event: {}", err)
        }
    }

    /// Dispatches registration or unregistration for all capability types.
    async fn sync_all_capabilities(
        &self,
        plugin_id: &str,
        instance: Option<Arc<Mutex<dyn PluginInstance>>>,
        caps: &PluginCapabilities,
    ) -> Result<(), PluginError> {
        // 1. Importers
        self.sync_registry(
            plugin_id,
            &self.importers_registry,
            &caps.importers,
            instance.as_ref(),
            |inst, cap| Arc::new(PluginImporterProxy::new(inst, cap)),
        )
        .await?;

        // 2. Updaters
        self.sync_registry(
            plugin_id,
            &self.updaters_registry,
            &caps.updaters,
            instance.as_ref(),
            |inst, cap| Arc::new(PluginUpdaterProxy::new(inst, cap)),
        )
        .await?;

        // 3. Content Providers
        self.sync_registry(
            plugin_id,
            &self.content_providers_registry,
            &caps.content_providers,
            instance.as_ref(),
            |inst, cap| Arc::new(PluginContentProviderProxy::new(inst, cap)),
        )
        .await?;

        Ok(())
    }

    /// Generic helper to add or remove items from any registry.
    /// If instance is Some -> Add, if None -> Remove.
    async fn sync_registry<T, C, R, F>(
        &self,
        plugin_id: &str,
        registry: &Arc<R>,
        items: &[C],
        instance: Option<&Arc<Mutex<dyn PluginInstance>>>,
        proxy_factory: F,
    ) -> Result<(), PluginError>
    where
        T: Send + Sync + ?Sized + 'static,
        R: CapabilityRegistry<Arc<T>> + ?Sized,
        C: Clone + AsCapabilityMetadata,
        F: Fn(Arc<Mutex<dyn PluginInstance>>, C) -> Arc<T>,
    {
        for capability in items {
            let meta = capability.as_metadata();

            match instance {
                Some(instance) => {
                    let proxy = proxy_factory(instance.clone(), capability.clone());
                    if let Err(err) = registry
                        .add(plugin_id.to_string(), meta.id.clone(), proxy)
                        .await
                    {
                        let error = PluginError::CapabilityRegistrationFailed {
                            capability_type: registry.get_type(),
                            capability_id: meta.id.clone(),
                        };
                        tracing::error!("{}: {}", error, err);
                    }
                }
                None => {
                    if let Err(err) = registry
                        .remove(plugin_id.to_string(), meta.id.clone())
                        .await
                    {
                        {
                            let error = PluginError::CapabilityCancelRegistrationFailed {
                                capability_type: registry.get_type(),
                                capability_id: meta.id.clone(),
                            };
                            tracing::error!("{}: {}", error, err);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
