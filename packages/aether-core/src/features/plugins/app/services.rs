use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::{
    multiple::RefMulti as DashMapRefMulti,
    one::{Ref as DashMapRef, RefMut as DashMapRefMut},
};
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::features::{
    events::{EventEmitterExt, PluginEvent, SharedEventEmitter},
    instance::PluginId,
    plugins::domain::LoadConfigType,
    plugins::{Plugin, PluginCapabilities, PluginError, PluginHandle, PluginManifest, PluginState},
};

use super::ports::{CommandApprover, PluginLoader};

pub struct PluginRegistry {
    plugins: DashMap<PluginId, Plugin>,
    settings_locks: DashMap<PluginId, Arc<Mutex<()>>>,
    event_emitter: SharedEventEmitter,
    /// Told when a plugin is stopped, so a question it left unanswered does not hold it.
    command_approver: Option<Arc<dyn CommandApprover>>,
}

/// How long a stop waits for a plugin to leave a command question it was cancelled out of.
const CANCEL_SETTLE: std::time::Duration = std::time::Duration::from_secs(5);

impl PluginRegistry {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self {
            plugins: DashMap::default(),
            settings_locks: DashMap::default(),
            event_emitter,
            command_approver: None,
        }
    }

    #[must_use]
    pub fn with_command_approver(mut self, approver: Arc<dyn CommandApprover>) -> Self {
        self.command_approver = Some(approver);
        self
    }

    /// Forgets the commands the user denied `plugin_id`. For a plugin that has really stopped: a
    /// stop refused, or one that did not retire the instance, leaves them in force.
    pub fn forget_command_denials(&self, plugin_id: &PluginId) {
        if let Some(approver) = &self.command_approver {
            approver.forget_denials(plugin_id);
        }
    }

    /// Cancels the command questions of `plugin_id` and waits for the call that asked them to
    /// end. A plugin waiting for the user is in a call, and a stop refuses a busy plugin; the
    /// cancelled question ends the call within moments. Plugins without a question pass at
    /// once.
    async fn cancel_command_questions(&self, plugin_id: &PluginId) {
        let Some(approver) = &self.command_approver else {
            return;
        };
        if !approver.cancel_plugin(plugin_id) {
            return;
        }
        let deadline = tokio::time::Instant::now() + CANCEL_SETTLE;
        while tokio::time::Instant::now() < deadline {
            let busy = self.plugins.get(plugin_id).is_some_and(
                |plugin| matches!(&plugin.state, PluginState::Loaded(handle) if handle.is_busy()),
            );
            if !busy {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }

    /// Held while a plugin's settings are read, changed and saved: every change rewrites the
    /// whole settings, so two at once would save over each other. Not to be held across a
    /// restart, which takes seconds and no settings change needs to wait for.
    pub async fn lock_settings(&self, plugin_id: &PluginId) -> OwnedMutexGuard<()> {
        let lock = Arc::clone(&self.settings_locks.entry(plugin_id.clone()).or_default());
        lock.lock_owned().await
    }

    pub fn insert(&self, plugin_id: PluginId, plugin: Plugin) {
        self.plugins.insert(plugin_id, plugin);
    }

    pub fn list(&self) -> impl Iterator<Item = DashMapRefMulti<'_, PluginId, Plugin>> {
        self.plugins.iter()
    }

    pub fn get(
        &self,
        plugin_id: &PluginId,
    ) -> Result<DashMapRef<'_, PluginId, Plugin>, PluginError> {
        self.plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound {
                plugin_id: plugin_id.to_string(),
            })
    }

    pub fn remove(&self, plugin_id: &PluginId) {
        self.plugins.remove(plugin_id);
    }

    pub fn get_ids(&self) -> HashSet<PluginId> {
        self.plugins
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn get_enabled_ids(&self) -> HashSet<PluginId> {
        self.plugins
            .iter()
            .filter_map(|entry| match &entry.state {
                PluginState::Loaded(_) => Some(entry.key().clone()),
                _ => None,
            })
            .collect()
    }

    pub fn list_manifests(&self) -> Result<Vec<PluginManifest>, PluginError> {
        Ok(self.list().map(|plugin| plugin.manifest.clone()).collect())
    }

    pub fn get_capabilities(
        &self,
        plugin_id: &PluginId,
    ) -> Result<Option<PluginCapabilities>, PluginError> {
        Ok(self.get(plugin_id)?.capabilities.clone())
    }

    pub fn get_manifest(&self, plugin_id: &PluginId) -> Result<PluginManifest, PluginError> {
        Ok(self.get(plugin_id)?.manifest.clone())
    }

    /// Clone state and manifest then drop the `DashMap` lock immediately.
    /// Prevents deadlocks when `upsert_with()` is called later.
    pub fn get_state_and_manifest(
        &self,
        plugin_id: &PluginId,
    ) -> Result<(PluginState, PluginManifest), PluginError> {
        let plugin = self.get(plugin_id)?;
        let state = plugin.state.clone();
        let manifest = plugin.manifest.clone();
        // Lock released on drop(plugin)
        Ok((state, manifest))
    }

    pub async fn upsert_with<F>(
        &self,
        plugin_id: &PluginId,
        update_fn: F,
    ) -> Result<(), PluginError>
    where
        F: FnOnce(&mut Plugin) -> Result<(), PluginError> + Send,
    {
        // The entry guard is released before the `await`: a handler that touches the same entry
        // would otherwise block on the guard we hold across the suspension point.
        {
            let mut plugin = self.get_mut(plugin_id)?;
            update_fn(&mut plugin)?;
        }

        self.event_emitter
            .emit_safe(PluginEvent::Edit {
                plugin_id: plugin_id.clone(),
            })
            .await;

        Ok(())
    }

    /// Takes the plugin to `Loading` in the same step that checks it may load, so that of two
    /// enables only one builds an instance; the other gets [`PluginError::LoadingInProgress`].
    /// Returns the state it was in, for a caller that gives up before loading, and its manifest.
    pub async fn begin_loading(
        &self,
        plugin_id: &PluginId,
    ) -> Result<(PluginState, PluginManifest), PluginError> {
        let mut taken = None;
        self.upsert_with(plugin_id, |plugin| {
            let plugin_id = plugin_id.clone();
            match &plugin.state {
                PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {}
                PluginState::Loading => return Err(PluginError::LoadingInProgress { plugin_id }),
                PluginState::Loaded(_) => return Err(PluginError::AlreadyLoaded { plugin_id }),
                PluginState::Unloading | PluginState::Removing => {
                    return Err(PluginError::UnloadingInProgress { plugin_id });
                }
            }
            let previous = std::mem::replace(&mut plugin.state, PluginState::Loading);
            taken = Some((previous, plugin.manifest.clone()));
            Ok(())
        })
        .await?;

        // The closure assigns `taken` before it returns `Ok`; every other path returned an error.
        Ok(taken.expect("the state is taken whenever it is changed"))
    }

    /// Puts back the state [`Self::begin_loading`] took the plugin from, unless something was
    /// decided about it since (it was found incompatible).
    pub async fn abandon_loading(
        &self,
        plugin_id: &PluginId,
        previous: PluginState,
    ) -> Result<(), PluginError> {
        self.upsert_with(plugin_id, |plugin| {
            if matches!(plugin.state, PluginState::Loading) {
                plugin.state = previous;
            }
            Ok(())
        })
        .await
    }

    /// Takes the plugin from `Loaded` to `Unloading` in one step, so that two restarts, or a
    /// restart and a disable, cannot both take the same instance. A busy plugin is refused
    /// without passing through `Unloading`.
    pub async fn begin_unloading(&self, plugin_id: &PluginId) -> Result<PluginHandle, PluginError> {
        self.cancel_command_questions(plugin_id).await;
        let mut taken = None;
        self.upsert_with(plugin_id, |plugin| {
            let plugin_id = plugin_id.clone();
            let handle = match &plugin.state {
                PluginState::Loaded(handle) => handle,
                PluginState::Loading => {
                    return Err(PluginError::LoadingInProgress { plugin_id });
                }
                PluginState::Unloading | PluginState::Removing => {
                    return Err(PluginError::UnloadingInProgress { plugin_id });
                }
                PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {
                    return Err(PluginError::AlreadyUnloaded { plugin_id });
                }
            };
            // The retirement checks this again for good.
            if handle.is_busy() {
                return Err(PluginError::Busy { plugin_id });
            }
            taken = Some(handle.clone());
            plugin.state = PluginState::Unloading;
            Ok(())
        })
        .await?;

        // The closure assigns `taken` before it returns `Ok`; every other path returned an error.
        Ok(taken.expect("the handle is taken whenever the state is changed"))
    }

    /// Takes a plugin that is not running to `Removing` in the step that checks nothing is
    /// loading it, so that of a remove and an enable only one goes on. Returns the state it
    /// was in, to put back should the delete fail. A plugin that is running, or in the middle of
    /// a transition, is refused with the matching error and its files are not to be touched.
    pub async fn begin_removing(&self, plugin_id: &PluginId) -> Result<PluginState, PluginError> {
        let mut taken = None;
        self.upsert_with(plugin_id, |plugin| {
            let plugin_id = plugin_id.clone();
            match &plugin.state {
                PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {}
                PluginState::Loading => return Err(PluginError::LoadingInProgress { plugin_id }),
                PluginState::Loaded(_) => return Err(PluginError::AlreadyLoaded { plugin_id }),
                PluginState::Unloading | PluginState::Removing => {
                    return Err(PluginError::UnloadingInProgress { plugin_id });
                }
            }
            taken = Some(std::mem::replace(&mut plugin.state, PluginState::Removing));
            Ok(())
        })
        .await?;

        Ok(taken.expect("the state is taken whenever it is changed"))
    }

    /// Puts back the state [`Self::begin_removing`] took the plugin from.
    pub async fn abandon_removing(
        &self,
        plugin_id: &PluginId,
        previous: PluginState,
    ) -> Result<(), PluginError> {
        self.upsert_with(plugin_id, |plugin| {
            if matches!(plugin.state, PluginState::Removing) {
                plugin.state = previous;
            }
            Ok(())
        })
        .await
    }

    /// The plugins whose files are being deleted.
    pub fn get_removing_ids(&self) -> HashSet<PluginId> {
        self.plugins
            .iter()
            .filter(|entry| matches!(entry.state, PluginState::Removing))
            .map(|entry| entry.key().clone())
            .collect()
    }

    fn get_mut(
        &self,
        plugin_id: &PluginId,
    ) -> Result<DashMapRefMut<'_, PluginId, Plugin>, PluginError> {
        self.plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound {
                plugin_id: plugin_id.to_string(),
            })
    }
}

#[derive(Default)]
pub struct PluginLoaderRegistry {
    loaders: HashMap<LoadConfigType, Arc<dyn PluginLoader>>,
}

impl PluginLoaderRegistry {
    pub fn new(loaders: HashMap<LoadConfigType, Arc<dyn PluginLoader>>) -> Self {
        Self { loaders }
    }

    pub fn get(
        &self,
        load_config_type: &LoadConfigType,
    ) -> Result<&Arc<dyn PluginLoader>, PluginError> {
        self.loaders
            .get(load_config_type)
            .ok_or(PluginError::LoaderNotFound {
                config_type: *load_config_type,
            })
    }

    pub fn register(&mut self, load_config_type: LoadConfigType, provider: Arc<dyn PluginLoader>) {
        self.loaders.insert(load_config_type, provider);
    }

    pub fn unregister(&mut self, load_config_type: &LoadConfigType) {
        self.loaders.remove(load_config_type);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::runtime::Handle;

    use super::*;
    use crate::features::{
        events::test_support::NoOpEmitter,
        plugins::{
            ApiConfig, CommandApprovalAnswer, CommandApprovalRefusal, CommandApprovalRequest,
            CommandGrant, CommandLine, LoadConfig, PluginInstance, PluginInternalEvent,
            PluginMetadata, RuntimeConfig,
        },
    };

    use crate::features::plugins::app::pending_command_approvals::PendingCommandApprovals;

    struct Stub;

    impl PluginInstance for Stub {
        fn get_id(&self) -> PluginId {
            id()
        }

        fn supports(&self, _name: &str) -> bool {
            true
        }

        fn call_bytes<'b>(
            &'b mut self,
            _name: &str,
            _args: &[u8],
        ) -> Result<&'b [u8], PluginError> {
            Ok(&[])
        }

        fn handle_event(&mut self, _event: &PluginInternalEvent) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn id() -> PluginId {
        PluginId::try_from("packwiz").unwrap()
    }

    fn manifest() -> PluginManifest {
        PluginManifest {
            metadata: PluginMetadata {
                id: id(),
                name: "Packwiz".into(),
                version: semver::Version::new(0, 1, 0),
                description: None,
                authors: vec![],
                license: None,
            },
            runtime: RuntimeConfig {
                allowed_hosts: vec![],
                allowed_paths: vec![],
            },
            load: LoadConfig::Extism {
                file: "plugin.wasm".try_into().unwrap(),
                memory_limit: None,
            },
            api: ApiConfig {
                version: semver::VersionReq::STAR,
                host_functions: Some(vec![]),
            },
        }
    }

    fn request(plugin: &PluginId) -> CommandApprovalRequest {
        CommandApprovalRequest {
            id: uuid::Uuid::new_v4(),
            plugin_id: plugin.clone(),
            java: "java".into(),
            current_dir: "/cache".into(),
            arguments: vec!["-jar".into(), "#/cache/a.jar".into()],
            line: CommandLine::default(),
            jar: None,
            always: Ok(()),
        }
    }

    struct Setup {
        registry: Arc<PluginRegistry>,
        approvals: Arc<PendingCommandApprovals>,
        handle: PluginHandle,
    }

    fn setup() -> Setup {
        let approvals = Arc::new(PendingCommandApprovals::new(Arc::new(NoOpEmitter)));
        let registry = Arc::new(
            PluginRegistry::new(Arc::new(NoOpEmitter))
                .with_command_approver(approvals.clone() as Arc<dyn CommandApprover>),
        );
        let handle = PluginHandle::new(Stub);
        registry.insert(
            id(),
            Plugin {
                manifest: manifest(),
                capabilities: None,
                hash: "hash".into(),
                verified: false,
                state: PluginState::Loaded(handle.clone()),
            },
        );
        Setup {
            registry,
            approvals,
            handle,
        }
    }

    /// A call into the plugin that asks the user, the way the `run_command` host function does:
    /// on the blocking pool, waiting on the approver.
    fn call_that_asks(
        setup: &Setup,
    ) -> tokio::task::JoinHandle<Result<Result<CommandGrant, CommandApprovalRefusal>, PluginError>>
    {
        let handle = setup.handle.clone();
        let approvals = setup.approvals.clone();
        tokio::spawn(async move {
            handle
                .run("import", move |_| {
                    Handle::current().block_on(approvals.request(request(&id())))
                })
                .await
        })
    }

    async fn wait_until_asked(approvals: &PendingCommandApprovals) {
        for _ in 0..500 {
            if !approvals.pending().is_empty() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("the plugin never asked");
    }

    #[tokio::test]
    async fn should_stop_the_plugin_when_it_waits_for_the_user_and_its_question_is_cancelled() {
        let setup = setup();
        let call = call_that_asks(&setup);
        wait_until_asked(&setup.approvals).await;

        let taken = setup.registry.begin_unloading(&id()).await;

        assert!(taken.is_ok(), "the stop should go through");
        let outcome = call.await.unwrap().expect("the call ended on its own");
        assert_eq!(outcome, Err(CommandApprovalRefusal::Cancelled));
        assert!(setup.approvals.pending().is_empty());
    }

    #[tokio::test]
    async fn should_refuse_to_stop_without_delay_when_the_busy_plugin_waits_for_nobody() {
        let setup = setup();
        let (release, gate) = std::sync::mpsc::channel::<()>();
        let handle = setup.handle.clone();
        let call = tokio::spawn(async move {
            handle
                .run("work", move |_| {
                    let _ = gate.recv();
                })
                .await
        });
        while !setup.handle.is_busy() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        let started = std::time::Instant::now();
        let result = setup.registry.begin_unloading(&id()).await;

        assert!(matches!(result, Err(PluginError::Busy { .. })));
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "with no question to cancel there is nothing to wait for"
        );
        release.send(()).unwrap();
        call.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn should_ask_again_when_the_registry_forgets_a_denial() {
        let setup = setup();
        let first = call_that_asks(&setup);
        wait_until_asked(&setup.approvals).await;
        let waiting = setup.approvals.pending()[0].request.id;
        setup
            .approvals
            .answer(waiting, CommandApprovalAnswer::Deny)
            .unwrap();
        assert_eq!(
            first.await.unwrap().unwrap(),
            Err(CommandApprovalRefusal::Denied)
        );

        let again = call_that_asks(&setup).await.unwrap().unwrap();
        assert_eq!(again, Err(CommandApprovalRefusal::Denied));

        setup.registry.forget_command_denials(&id());
        let asked_anew = call_that_asks(&setup);
        wait_until_asked(&setup.approvals).await;
        setup.approvals.cancel_plugin(&id());
        assert_eq!(
            asked_anew.await.unwrap().unwrap(),
            Err(CommandApprovalRefusal::Cancelled)
        );
    }
}
