use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::{
    features::{
        events::ProgressService,
        instance::{ContentDownloader, InstanceError, ProviderId},
        plugins::{PluginRegistry, PluginSettingsStorage},
    },
    shared::{
        fetch::FetchSemaphore,
        hash::FileHash,
        limits::{ContentInstallLimits, PluginSandboxLimits},
        request_client::{BytesProgress, Request, RequestClient},
    },
};

use super::{
    extism::{
        AddressFilter, PluginNetwork, ProxySource, VettedDownloadClient, download_with,
        is_blocked_ip, vetted_download_client,
    },
    plugin_utils::effective_allowed_hosts,
};

/// Downloads the files of an install plan by who supplied their URL: a built-in provider's
/// through the application's client, a plugin's through the vetted client of its host
/// functions, limited to that plugin's `allowed_hosts`, since a plugin's URL is the plugin's word.
///
/// Built-in means the reserved `aether` plugin id ([`ProviderId::is_builtin`]), which no plugin
/// can take — not the host a URL points at.
///
/// A plugin's client is kept while its `allowed_hosts` stay the same, so a plan's files share
/// connections; one downloader serves one install, so a kept client never outlives it.
pub struct ProviderContentDownloader {
    /// For built-in providers only: it follows redirects unchecked.
    app_client: Arc<dyn RequestClient>,
    plugins: Arc<PluginRegistry>,
    plugin_settings: Arc<dyn PluginSettingsStorage>,
    semaphore: Arc<FetchSemaphore>,
    progress_service: Arc<dyn ProgressService>,
    proxy: ProxySource,
    is_blocked: AddressFilter,
    content_limits: ContentInstallLimits,
    sandbox_limits: PluginSandboxLimits,
    /// By plugin id: the client and the `allowed_hosts` it was built for.
    clients: Mutex<HashMap<String, PluginClient>>,
}

struct PluginClient {
    allowed_hosts: Vec<String>,
    client: Arc<VettedDownloadClient>,
}

impl ProviderContentDownloader {
    pub fn new(
        app_client: Arc<dyn RequestClient>,
        plugins: Arc<PluginRegistry>,
        plugin_settings: Arc<dyn PluginSettingsStorage>,
        semaphore: Arc<FetchSemaphore>,
        progress_service: Arc<dyn ProgressService>,
        content_limits: ContentInstallLimits,
        sandbox_limits: PluginSandboxLimits,
    ) -> Self {
        Self {
            app_client,
            plugins,
            plugin_settings,
            semaphore,
            progress_service,
            proxy: ProxySource::System,
            is_blocked: is_blocked_ip,
            content_limits,
            sandbox_limits,
            clients: Mutex::default(),
        }
    }

    /// The same downloader with a loopback-friendly network, for tests that need a local server
    /// to stand in for a plugin's host.
    #[cfg(test)]
    pub(crate) fn with_test_network(mut self, is_blocked: AddressFilter) -> Self {
        self.proxy = ProxySource::Direct;
        self.is_blocked = is_blocked;
        self
    }

    fn failed(url: &str, err: impl std::fmt::Display) -> InstanceError {
        InstanceError::ContentDownloadError(format!("{url}: {err}"))
    }

    /// What the plugin may reach now: its manifest and the user's current settings. A plugin
    /// that is gone has no policy, so its URLs are not downloaded at all.
    async fn plugin_allowed_hosts(&self, plugin_id: &str) -> Result<Vec<String>, InstanceError> {
        let manifest = self.plugins.get_manifest(plugin_id).map_err(|err| {
            InstanceError::ContentDownloadError(format!(
                "No network policy for plugin \"{plugin_id}\": {err}"
            ))
        })?;
        let settings = self
            .plugin_settings
            .get(plugin_id)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        Ok(effective_allowed_hosts(&manifest, settings.as_ref()))
    }

    /// The plugin's client for `allowed_hosts`: the kept one, or a new one when there is none
    /// yet or the hosts changed since it was built.
    fn plugin_client(
        &self,
        plugin_id: &str,
        allowed_hosts: &[String],
    ) -> Result<Arc<VettedDownloadClient>, String> {
        let mut clients = self
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(kept) = clients.get(plugin_id)
            && kept.allowed_hosts == allowed_hosts
        {
            return Ok(kept.client.clone());
        }

        let network = PluginNetwork {
            allowed_hosts,
            semaphore: self.semaphore.clone(),
            progress_service: self.progress_service.clone(),
            proxy: &self.proxy,
            is_blocked: self.is_blocked,
            limits: self.sandbox_limits,
        };
        let client = Arc::new(vetted_download_client(&network).map_err(|err| err.to_string())?);
        clients.insert(
            plugin_id.to_owned(),
            PluginClient {
                allowed_hosts: allowed_hosts.to_vec(),
                client: client.clone(),
            },
        );

        Ok(client)
    }
}

#[async_trait]
impl ContentDownloader for ProviderContentDownloader {
    async fn download(
        &self,
        origin: &ProviderId,
        url: &str,
        hash: Option<&FileHash>,
        destination: &Path,
        on_bytes: Option<BytesProgress<'_>>,
    ) -> Result<u64, InstanceError> {
        if origin.is_builtin() {
            let mut request = Request::get(url);
            if let Some(hash) = hash {
                request = request.with_hash(hash.clone());
            }

            return self
                .app_client
                .download_to_file(
                    request,
                    destination,
                    self.content_limits.max_content_bytes,
                    on_bytes,
                )
                .await
                .map_err(|err| Self::failed(url, err));
        }

        // Read for every file: the user may narrow the plugin's hosts while a plan downloads.
        let allowed_hosts = self.plugin_allowed_hosts(origin.plugin_id.as_str()).await?;
        let client = self
            .plugin_client(origin.plugin_id.as_str(), &allowed_hosts)
            .map_err(|err| Self::failed(url, err))?;

        download_with(
            &client,
            &allowed_hosts,
            url,
            hash.cloned(),
            destination,
            self.sandbox_limits.max_download_bytes,
            on_bytes,
        )
        .await
        .map_err(|err| {
            tracing::warn!("Download of {url} for provider \"{origin}\" failed: {err}");
            Self::failed(url, err)
        })
    }
}
