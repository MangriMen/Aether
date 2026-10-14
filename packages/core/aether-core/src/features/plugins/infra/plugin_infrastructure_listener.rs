use std::sync::Arc;

use crate::{
    features::{
        events::PluginEvent,
        instance::{ContentProvider, Importer, Updater},
        plugins::{
            AsCapabilityMetadata, PluginCapabilities, PluginError, PluginHandle, PluginRegistry,
            PluginState,
        },
        settings::LocationInfo,
    },
    shared::capability::domain::CapabilityRegistry,
};

use super::{PluginContentProviderProxy, PluginImporterProxy, PluginUpdaterProxy};

pub struct PluginInfrastructureListener<
    IR: ?Sized + CapabilityRegistry<Arc<dyn Importer>>,
    UR: ?Sized + CapabilityRegistry<Arc<dyn Updater>>,
    CR: ?Sized + CapabilityRegistry<Arc<dyn ContentProvider>>,
> {
    plugin: Arc<PluginRegistry>,
    importers: Arc<IR>,
    updaters: Arc<UR>,
    content_providers: Arc<CR>,
    /// For content provider proxies, which translate the paths a plugin names.
    location_info: Arc<LocationInfo>,
}

impl<IR, UR, CR> PluginInfrastructureListener<IR, UR, CR>
where
    IR: ?Sized + CapabilityRegistry<Arc<dyn Importer>>,
    UR: ?Sized + CapabilityRegistry<Arc<dyn Updater>>,
    CR: ?Sized + CapabilityRegistry<Arc<dyn ContentProvider>>,
{
    pub fn new(
        plugin: Arc<PluginRegistry>,
        importers: Arc<IR>,
        updaters: Arc<UR>,
        content_providers: Arc<CR>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            plugin,
            importers,
            updaters,
            content_providers,
            location_info,
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

            // Passing states keep the registrations: during an unload or restart the proxies
            // answer "not running" over the retired handle instead of the capability going
            // missing, until the new instance or the final state replaces them.
            if matches!(state, PluginState::Loading | PluginState::Unloading) {
                return Ok(());
            }

            let capabilities = self.plugin.get_capabilities(&plugin_id)?;

            if let Some(caps) = capabilities {
                match state {
                    PluginState::Loaded(instance) => {
                        self.sync_all_capabilities(&plugin_id, Some(instance), &caps)
                            .await?;
                    }
                    PluginState::NotLoaded
                    | PluginState::Failed(_)
                    | PluginState::Incompatible(_) => {
                        self.sync_all_capabilities(&plugin_id, None, &caps).await?;
                    }
                    PluginState::Loading | PluginState::Unloading => (),
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
        instance: Option<PluginHandle>,
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
            |inst, cap| {
                Arc::new(PluginContentProviderProxy::new(
                    plugin_id.to_owned(),
                    inst,
                    cap,
                    self.location_info.clone(),
                ))
            },
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
        instance: Option<&PluginHandle>,
        proxy_factory: F,
    ) -> Result<(), PluginError>
    where
        T: Send + Sync + ?Sized + 'static,
        R: CapabilityRegistry<Arc<T>> + ?Sized,
        C: Clone + AsCapabilityMetadata,
        F: Fn(PluginHandle, C) -> Arc<T>,
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
