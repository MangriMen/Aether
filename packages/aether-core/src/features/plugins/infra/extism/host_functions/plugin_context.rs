use std::sync::{Arc, Weak};

use async_trait::async_trait;

use crate::{
    core::app::AetherContainer,
    features::{
        instance::PluginId,
        plugins::{
            PluginError, PluginRegistry, PluginSettingsStorage,
            infra::plugin_utils::effective_allowed_hosts,
        },
    },
    shared::limits::PluginSandboxLimits,
};

/// Where a plugin's network policy comes from, asked on every host function call.
///
/// The list is deliberately not copied into the context when the plugin loads: the user can
/// change a plugin's hosts in its settings while it is running, and the change must apply to
/// its very next request without a reload.
#[async_trait]
pub(crate) trait AllowedHostsSource: Send + Sync {
    async fn allowed_hosts(&self, plugin_id: &PluginId) -> crate::Result<Vec<String>>;
}

/// The manifest in the registry plus the user's settings — what [`effective_allowed_hosts`]
/// merges. A plugin that is gone has no policy, so it gets an error rather than an open list.
pub(crate) struct LiveAllowedHosts {
    registry: Arc<PluginRegistry>,
    settings: Arc<dyn PluginSettingsStorage>,
}

#[async_trait]
impl AllowedHostsSource for LiveAllowedHosts {
    async fn allowed_hosts(&self, plugin_id: &PluginId) -> crate::Result<Vec<String>> {
        let to_error = |err: PluginError| crate::ErrorKind::from(err).as_error();

        let manifest = self.registry.get_manifest(plugin_id).map_err(to_error)?;
        let settings = self.settings.get(plugin_id).await.map_err(to_error)?;

        Ok(effective_allowed_hosts(&manifest, settings.as_ref()))
    }
}

#[derive(Clone)]
pub(crate) struct PluginContext {
    pub id: PluginId,
    pub container: Weak<AetherContainer>,
    /// The plugin's network policy, read on each call. Host functions that touch the network
    /// check against it themselves instead of trusting Extism's own check (see
    /// `features::http`); the Extism manifest carries no hosts at all, so its built-in
    /// `http_request` refuses everything.
    pub hosts: Arc<dyn AllowedHostsSource>,
    /// The sandbox limits the container was built with, taken once here so a host function
    /// does not have to reach back into the container for them.
    pub limits: PluginSandboxLimits,
}

impl PluginContext {
    pub(crate) fn new(id: PluginId, container: &Arc<AetherContainer>) -> Self {
        let plugins = container.plugins();

        Self {
            id,
            container: Arc::downgrade(container),
            hosts: Arc::new(LiveAllowedHosts {
                registry: plugins.registry.clone(),
                settings: plugins.settings_storage.clone(),
            }),
            limits: container.sandbox_limits(),
        }
    }

    pub(crate) fn upgrade_container(&self) -> Option<Arc<AetherContainer>> {
        self.container.upgrade()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use aether_core_plugin_api::v0::{HostResult, HttpRequestDto, HttpResponseDto};
    use extism::{Function, Manifest, Plugin, PluginBuilder, UserData, Wasm};

    use super::*;

    /// A loopback target: a host that passes the plugin's list reaches the address filter and
    /// is refused there (no public address), which tells "allowed" from "not listed" without
    /// any network. Two literals stand for two different hosts.
    const FIRST: &str = "127.0.0.1";
    const SECOND: &str = "127.0.0.2";

    /// Exports `via_http_get`, which hands its input to the core's `http_get` and returns the
    /// answer, and `via_builtin`, which sends its input to Extism's own `http_request`.
    const PLUGIN: &str = r#"(module
        (import "extism:host/env" "input_offset" (func $input_offset (result i64)))
        (import "extism:host/env" "length" (func $length (param i64) (result i64)))
        (import "extism:host/env" "output_set" (func $output_set (param i64 i64)))
        (import "extism:host/env" "http_request" (func $http_request (param i64 i64) (result i64)))
        (import "extism:host/user" "http_get" (func $http_get (param i64) (result i64)))
        (func (export "via_http_get") (result i32)
            (local $out i64)
            (local.set $out (call $http_get (call $input_offset)))
            (call $output_set (local.get $out) (call $length (local.get $out)))
            (i32.const 0))
        (func (export "via_builtin") (result i32)
            (drop (call $http_request (call $input_offset) (i64.const 0)))
            (i32.const 0)))"#;

    /// The list as the user's settings hold it right now; the test edits it between calls.
    struct EditableHosts(Mutex<Vec<String>>);

    #[async_trait]
    impl AllowedHostsSource for EditableHosts {
        async fn allowed_hosts(&self, _plugin_id: &PluginId) -> crate::Result<Vec<String>> {
            Ok(self.0.lock().unwrap().clone())
        }
    }

    /// The plugin as the loader builds it: no `allowed_hosts` in the Extism manifest.
    fn plugin(hosts: Arc<EditableHosts>) -> Plugin {
        let context = PluginContext {
            id: PluginId::from_static("test"),
            container: Weak::new(),
            hosts,
            limits: PluginSandboxLimits::default(),
        };
        let http_get = Function::new(
            "http_get",
            [extism::PTR],
            [extism::PTR],
            UserData::new(context),
            super::super::features::http_get,
        );

        PluginBuilder::new(Manifest::new([Wasm::data(PLUGIN.as_bytes().to_vec())]))
            .with_functions([http_get])
            .with_wasi(true)
            .build()
            .expect("the test plugin should load")
    }

    fn get(plugin: &mut Plugin, host: &str) -> String {
        let request = HttpRequestDto {
            url: format!("http://{host}/"),
            headers: HashMap::new(),
        };
        let input = rmp_serde::to_vec(&request).expect("encodes");

        let output: Vec<u8> = plugin
            .call("via_http_get", input)
            .expect("http_get answers with a result, not a trap");
        let answer: HostResult<HttpResponseDto> = rmp_serde::from_slice(&output).expect("decodes");

        format!("{answer:?}")
    }

    /// Runs `check` where a plugin call may run: off the runtime workers, with the runtime
    /// at hand for the host function to block on.
    async fn on_blocking_pool<T: Send + 'static>(check: impl FnOnce() -> T + Send + 'static) -> T {
        tokio::task::spawn_blocking(check).await.expect("joins")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_refuse_extisms_own_http_request_when_the_host_is_listed() {
        let hosts = Arc::new(EditableHosts(Mutex::new(vec![FIRST.to_owned()])));

        let error = on_blocking_pool(move || {
            let mut plugin = plugin(hosts);
            let request = format!(r#"{{"url":"http://{FIRST}/","method":"GET"}}"#);

            plugin
                .call::<_, Vec<u8>>("via_builtin", request)
                .expect_err("the built-in request must be refused")
                .root_cause()
                .to_string()
        })
        .await;

        assert!(error.contains("is not allowed"), "{error}");
        assert!(!error.contains("not enabled"), "{error}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_follow_the_hosts_list_when_it_changes_without_reloading_the_plugin() {
        let hosts = Arc::new(EditableHosts(Mutex::new(vec![FIRST.to_owned()])));
        let edited = hosts.clone();

        let (listed, unlisted, after_edit_new, after_edit_removed) = on_blocking_pool(move || {
            let mut plugin = plugin(hosts);
            let listed = get(&mut plugin, FIRST);
            let unlisted = get(&mut plugin, SECOND);

            *edited.0.lock().unwrap() = vec![SECOND.to_owned()];

            let after_edit_new = get(&mut plugin, SECOND);
            let after_edit_removed = get(&mut plugin, FIRST);
            (listed, unlisted, after_edit_new, after_edit_removed)
        })
        .await;

        // Past the host check, stopped by the address filter: the host was allowed.
        assert!(listed.contains("non-public"), "{listed}");
        assert!(unlisted.contains("allowed_hosts"), "{unlisted}");
        assert!(after_edit_new.contains("non-public"), "{after_edit_new}");
        assert!(
            after_edit_removed.contains("allowed_hosts"),
            "{after_edit_removed}"
        );
    }
}
