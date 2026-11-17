use std::{net::IpAddr, sync::Arc};

use async_trait::async_trait;
use futures::future::BoxFuture;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    AllowedHostRejection, LocalHost, PluginError, PluginRegistry, PluginSettingsStorage,
};
use crate::shared::public_address::{is_blocked_ip, is_never_grantable};

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
///
/// The plugin names the host the grant is offered for (in the refusal it reports), so a DNS
/// name is resolved here, dry run or not, and refused unless every address it has is a local
/// one: the confirmation tells the user it is in their own network, and that must be true. A
/// granted name still has to be in the plugin's `allowed_hosts` to be reached at all
/// ([`PluginHosts::admits`](crate::features::plugins::PluginHosts::admits)).
pub(crate) struct GrantPluginLocalHostAccessUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    lookup: NameLookup,
}

/// Resolves a host name to its addresses. The operating system's resolver in production;
/// tests substitute a fixed answer.
pub(crate) type NameLookup = fn(String, u16) -> BoxFuture<'static, std::io::Result<Vec<IpAddr>>>;

fn system_lookup(host: String, port: u16) -> BoxFuture<'static, std::io::Result<Vec<IpAddr>>> {
    Box::pin(async move {
        Ok(tokio::net::lookup_host((host.as_str(), port))
            .await?
            .map(|addr| addr.ip())
            .collect())
    })
}

impl GrantPluginLocalHostAccessUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
            lookup: system_lookup,
        }
    }

    /// The same use case resolving names with `lookup`.
    #[cfg(test)]
    pub(crate) fn with_lookup(mut self, lookup: NameLookup) -> Self {
        self.lookup = lookup;
        self
    }

    /// Refuses a DNS name that does not resolve, or resolves to an address that is public or
    /// no single host. A literal was judged by [`LocalHost::from_url`]; `localhost` is local.
    async fn check_resolves_locally(&self, local: &LocalHost) -> Result<(), AllowedHostRejection> {
        if local.is_local_by_name() {
            return Ok(());
        }
        let addrs = (self.lookup)(local.host().to_owned(), local.port().unwrap_or(80))
            .await
            .map_err(|_| AllowedHostRejection::Unresolved)?;
        if addrs.is_empty() {
            return Err(AllowedHostRejection::Unresolved);
        }
        if addrs
            .iter()
            .any(|&ip| !is_blocked_ip(ip) || is_never_grantable(ip))
        {
            return Err(AllowedHostRejection::NotLocal);
        }
        Ok(())
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: PluginId,
        url: String,
        dry_run: bool,
    ) -> Result<HostAccess, PluginError> {
        // The plugin must exist: a grant is for one plugin, never stored for a stray id.
        self.plugin_registry.get_manifest(&plugin_id)?;
        let invalid = |reason| PluginError::InvalidLocalHost {
            plugin_id: plugin_id.clone(),
            url: url.clone(),
            reason,
        };
        let local = LocalHost::from_url(&url).map_err(invalid)?;
        self.check_resolves_locally(&local).await.map_err(invalid)?;
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
