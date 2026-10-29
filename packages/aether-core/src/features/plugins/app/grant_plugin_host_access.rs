use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    GrantableHost, PluginError, PluginRegistry, PluginSettingsStorage, effective_allowed_hosts,
    is_host_allowed,
};

use super::{
    plugin_access_grant::{SettingsGrant, grant_in_settings},
    ports::GrantPluginHostAccessUseCasePort,
};

/// What became of the host of a URL the user gave a plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostAccess {
    /// The plugin could already reach the host: through its manifest, or an entry of the user's.
    AlreadyAllowed { host: String },
    /// A dry run: the plugin cannot reach the host yet, and nothing was changed.
    Needed { host: String },
    /// The host was added to the plugin's `allowed_hosts`.
    Granted { host: String },
}

/// Lets a plugin reach the host of a URL the user handed it (a pack an importer is to download).
/// A dry run tells the UI whether there is anything to ask the user.
///
/// The entry added admits exactly the host the user saw, matched as the plugin's requests match
/// it ([`is_host_allowed`]); hosts those requests would refuse anyway are refused here.
/// No restart is needed: the network host functions read `allowed_hosts` on every call.
pub(crate) struct GrantPluginHostAccessUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl GrantPluginHostAccessUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
        }
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: PluginId,
        url: String,
        dry_run: bool,
    ) -> Result<HostAccess, PluginError> {
        let manifest = self.plugin_registry.get_manifest(&plugin_id)?;
        let GrantableHost { host, entry } =
            GrantableHost::from_url(&url).map_err(|reason| PluginError::InvalidAllowedHost {
                plugin_id: plugin_id.clone(),
                url: url.clone(),
                reason,
            })?;

        let grant = grant_in_settings(
            &self.plugin_registry,
            self.plugin_settings_storage.as_ref(),
            &plugin_id,
            dry_run,
            |settings| is_host_allowed(&host, &effective_allowed_hosts(&manifest, Some(settings))),
            |settings| settings.allowed_hosts.push(entry),
        )
        .await?;

        Ok(match grant {
            SettingsGrant::AlreadyAllowed => HostAccess::AlreadyAllowed { host },
            SettingsGrant::Needed => HostAccess::Needed { host },
            SettingsGrant::Granted => HostAccess::Granted { host },
        })
    }
}

#[async_trait]
impl GrantPluginHostAccessUseCasePort for GrantPluginHostAccessUseCase {
    async fn execute(
        &self,
        plugin_id: PluginId,
        url: String,
        dry_run: bool,
    ) -> Result<HostAccess, PluginError> {
        self.execute(plugin_id, url, dry_run).await
    }
}
