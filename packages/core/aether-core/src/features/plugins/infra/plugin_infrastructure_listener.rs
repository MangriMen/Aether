use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    features::{
        events::PluginEvent,
        instance::{ContentProvider, Importer, Updater},
        plugins::{
            AsCapabilityMetadata, PluginCapabilities, PluginError, PluginInstance, PluginRegistry,
            PluginState,
            infra::{PluginContentProviderProxy, PluginImporterProxy, PluginUpdaterProxy},
        },
    },
    shared::CapabilityRegistry,
};

pub struct PluginInfrastructureListener<
    IR: CapabilityRegistry<Arc<dyn Importer>>,
    UR: CapabilityRegistry<Arc<dyn Updater>>,
    CR: CapabilityRegistry<Arc<dyn ContentProvider>>,
> {
    plugin: Arc<PluginRegistry>,
    importers: Arc<IR>,
    updaters: Arc<UR>,
    content_providers: Arc<CR>,
}

impl<IR, UR, CR> PluginInfrastructureListener<IR, UR, CR>
where
    IR: CapabilityRegistry<Arc<dyn Importer>>,
    UR: CapabilityRegistry<Arc<dyn Updater>>,
    CR: CapabilityRegistry<Arc<dyn ContentProvider>>,
{
    pub fn new(
        plugin: Arc<PluginRegistry>,
        importers: Arc<IR>,
        updaters: Arc<UR>,
        content_providers: Arc<CR>,
    ) -> Self {
        Self {
            plugin,
            importers,
            updaters,
            content_providers,
        }
    }

    pub async fn on_plugin_event(&self, event: PluginEvent) {
        let result: Result<(), PluginError> = async {
            let PluginEvent::Edit { plugin_id } = event else {
                return Ok(());
            };

            let state = {
                let plugin = self.plugin.get(&plugin_id)?;
                plugin.state.clone()
            };

            if matches!(state, PluginState::Loading) {
                return Ok(());
            }

            let capabilities = self.plugin.get_capabilities(&plugin_id)?;

            if let Some(caps) = capabilities {
                match state {
                    PluginState::Loaded(instance) => {
                        self.sync_all_capabilities(&plugin_id, Some(instance), &caps)
                            .await?;
                    }
                    PluginState::NotLoaded | PluginState::Unloading | PluginState::Failed(_) => {
                        self.sync_all_capabilities(&plugin_id, None, &caps).await?;
                    }
                    PluginState::Loading => (),
                }
            }

            Ok(())
        }
        .await;

        if let Err(err) = result {
            tracing::error!("Error handling plugin event: {}", err);
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
            &self.importers,
            &caps.importers,
            instance.as_ref(),
            |inst, cap| Arc::new(PluginImporterProxy::new(inst, cap)),
        )
        .await?;

        // 2. Updaters
        self.sync_registry(
            plugin_id,
            &self.updaters,
            &caps.updaters,
            instance.as_ref(),
            |inst, cap| Arc::new(PluginUpdaterProxy::new(inst, cap)),
        )
        .await?;

        // 3. Content Providers
        self.sync_registry(
            plugin_id,
            &self.content_providers,
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
