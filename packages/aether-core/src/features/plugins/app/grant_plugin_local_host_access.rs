use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{LocalHost, PluginError, PluginRegistry, PluginSettingsStorage};

use super::{
    grant_plugin_host_access::HostAccess,
    plugin_access_grant::{SettingsGrant, grant_in_settings},
    ports::GrantPluginLocalHostAccessUseCasePort,
};

/// Lets a plugin reach the host of a URL, and that URL's port, at a non-public address: a
/// server in the user's own network, or on this machine. A dry run tells the UI whether there
/// is anything to ask the user.
///
/// Only ever the user's decision, taken behind a confirmation that says what it gives: no
/// manifest, and nothing a plugin sends, reaches this. The grant names one host and port as
/// the plugin's requests name them, never a pattern ([`LocalHost::from_url`]); the plugin's
/// requests are still pinned to the addresses checked for each one, and a redirect to a local
/// address that is not granted is still refused. No restart is needed: the network host
/// functions read the plugin's hosts on every call.
pub(crate) struct GrantPluginLocalHostAccessUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl GrantPluginLocalHostAccessUseCase {
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
        // The plugin must exist: a grant is for one plugin, never stored for a stray id.
        self.plugin_registry.get_manifest(&plugin_id)?;
        let local =
            LocalHost::from_url(&url).map_err(|reason| PluginError::InvalidAllowedHost {
                plugin_id: plugin_id.clone(),
                url: url.clone(),
                reason,
            })?;
        let host = local.to_string();
        let port = local.port().unwrap_or_default();

        let grant = grant_in_settings(
            &self.plugin_registry,
            self.plugin_settings_storage.as_ref(),
            &plugin_id,
            dry_run,
            |settings| {
                settings
                    .local_hosts
                    .iter()
                    .any(|granted| granted.admits(local.host(), port))
            },
            |settings| settings.local_hosts.push(local.clone()),
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
impl GrantPluginLocalHostAccessUseCasePort for GrantPluginLocalHostAccessUseCase {
    async fn execute(
        &self,
        plugin_id: PluginId,
        url: String,
        dry_run: bool,
    ) -> Result<HostAccess, PluginError> {
        self.execute(plugin_id, url, dry_run).await
    }
}
