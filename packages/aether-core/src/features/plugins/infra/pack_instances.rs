//! Which instances a plugin creates for a pack while it deploys one.
//!
//! A plugin that installs a pack itself makes the instance through `instance_create`, which
//! installs the game; without more the instance would be `Installed`, and launchable without its
//! mods, while the plugin is still downloading them. So an instance created with `pack_info`
//! during a deployment ([`deploying_packs`]) is created `PackInstalling` and recorded, and the
//! caller moves it on once the call has succeeded.
//!
//! A host function runs on the thread of the call it serves (`PluginHandle::run`), so the
//! deployment is a thread-local, and only the plugin that opened it records into it.

use std::cell::RefCell;

use crate::features::instance::{InstanceId, PluginId};

thread_local! {
    static DEPLOYMENT: RefCell<Option<Deployment>> = const { RefCell::new(None) };
}

struct Deployment {
    plugin_id: PluginId,
    instances: Vec<InstanceId>,
}

/// Run `f` — a call into `plugin_id` — with a deployment open, and give back the instances the
/// plugin created for a pack meanwhile.
pub(crate) fn deploying_packs<R>(
    plugin_id: &PluginId,
    f: impl FnOnce() -> R,
) -> (R, Vec<InstanceId>) {
    let scope = DeploymentScope(DEPLOYMENT.replace(Some(Deployment {
        plugin_id: plugin_id.clone(),
        instances: Vec::new(),
    })));

    let result = f();
    let instances = DEPLOYMENT
        .with_borrow_mut(|deployment| {
            deployment
                .as_mut()
                .map(|d| std::mem::take(&mut d.instances))
        })
        .unwrap_or_default();
    drop(scope);

    (result, instances)
}

/// Whether an instance `plugin_id` creates now is created for a pack it is deploying.
pub(crate) fn is_deploying_packs(plugin_id: &PluginId) -> bool {
    DEPLOYMENT.with_borrow(|deployment| {
        deployment
            .as_ref()
            .is_some_and(|deployment| &deployment.plugin_id == plugin_id)
    })
}

/// Record an instance `plugin_id` created for a pack; nothing when it deploys none.
pub(crate) fn record_pack_instance(plugin_id: &PluginId, instance_id: &InstanceId) {
    DEPLOYMENT.with_borrow_mut(|deployment| {
        if let Some(deployment) = deployment
            .as_mut()
            .filter(|deployment| &deployment.plugin_id == plugin_id)
        {
            deployment.instances.push(instance_id.clone());
        }
    });
}

/// Puts back the deployment that was open before, even when the call panics: the blocking-pool
/// thread goes on to run other calls.
struct DeploymentScope(Option<Deployment>);

impl Drop for DeploymentScope {
    fn drop(&mut self) {
        DEPLOYMENT.set(self.0.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(id: &str) -> InstanceId {
        InstanceId::try_from(id).unwrap()
    }

    #[test]
    fn should_collect_the_pack_instances_when_the_deploying_plugin_creates_them() {
        let (result, instances) = deploying_packs(&PluginId::from_static("packwiz"), || {
            assert!(is_deploying_packs(&PluginId::from_static("packwiz")));
            record_pack_instance(&PluginId::from_static("packwiz"), &id("a"));
            record_pack_instance(&PluginId::from_static("packwiz"), &id("b"));
            42
        });

        assert_eq!(result, 42);
        assert_eq!(instances, [id("a"), id("b")]);
        assert!(!is_deploying_packs(&PluginId::from_static("packwiz")));
    }

    /// A plugin the call leads to deploys nothing of the caller's.
    #[test]
    fn should_ignore_another_plugin_when_it_creates_pack_instances() {
        let ((), instances) = deploying_packs(&PluginId::from_static("packwiz"), || {
            assert!(!is_deploying_packs(&PluginId::from_static("other")));
            record_pack_instance(&PluginId::from_static("other"), &id("x"));
        });

        assert!(instances.is_empty());
    }

    #[test]
    fn should_record_nothing_when_outside_a_deployment() {
        assert!(!is_deploying_packs(&PluginId::from_static("packwiz")));
        record_pack_instance(&PluginId::from_static("packwiz"), &id("a"));

        let ((), instances) = deploying_packs(&PluginId::from_static("packwiz"), || {});
        assert!(instances.is_empty());
    }

    /// A deployment opened inside another gives the outer one back when it ends.
    #[test]
    fn should_restore_the_outer_deployment_when_a_nested_one_ends() {
        let ((), outer) = deploying_packs(&PluginId::from_static("a"), || {
            let ((), inner) = deploying_packs(&PluginId::from_static("b"), || {
                record_pack_instance(&PluginId::from_static("b"), &id("inner"));
            });
            assert_eq!(inner, [id("inner")]);
            record_pack_instance(&PluginId::from_static("a"), &id("outer"));
        });

        assert_eq!(outer, [id("outer")]);
    }

    #[test]
    fn should_close_the_deployment_when_the_call_panics() {
        let panicked = std::panic::catch_unwind(|| {
            deploying_packs(&PluginId::from_static("packwiz"), || panic!("plugin call"));
        });

        assert!(panicked.is_err());
        assert!(!is_deploying_packs(&PluginId::from_static("packwiz")));
    }
}
