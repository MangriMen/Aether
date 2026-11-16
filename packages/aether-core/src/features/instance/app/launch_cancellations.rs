use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio_util::sync::CancellationToken;

use crate::features::instance::InstanceId;

/// The launches in progress, by instance, so that the user can call one off while it has no
/// process to stop yet (it waits for the pre-launch hook, say).
///
/// Apart from [`InstallCancellations`](super::InstallCancellations): that one is called off by
/// the removal of an instance, this one by the user, and a launch not found is no error.
#[derive(Default)]
pub struct LaunchCancellations {
    launches: Mutex<HashMap<InstanceId, (u64, CancellationToken)>>,
    next: AtomicU64,
}

/// A launch, in the registry until dropped.
#[must_use = "the launch is out of the registry as soon as this is dropped"]
pub struct RegisteredLaunch {
    registry: Arc<LaunchCancellations>,
    instance_id: InstanceId,
    key: u64,
    /// Fires when the user calls the launch off.
    pub token: CancellationToken,
}

impl LaunchCancellations {
    fn launches(&self) -> MutexGuard<'_, HashMap<InstanceId, (u64, CancellationToken)>> {
        self.launches.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Registers a launch of the instance. An instance is launched once at a time, so a launch
    /// that is registered already is replaced; its late drop leaves the new one in.
    pub fn register(self: &Arc<Self>, instance_id: &InstanceId) -> RegisteredLaunch {
        let key = self.next.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();
        self.launches()
            .insert(instance_id.clone(), (key, token.clone()));
        RegisteredLaunch {
            registry: Arc::clone(self),
            instance_id: instance_id.clone(),
            key,
            token,
        }
    }

    /// Calls off the launch of the instance. Whether there was one: a click may come before the
    /// launch is registered, or after the game has started, and then the caller stops the game.
    pub fn cancel(&self, instance_id: &InstanceId) -> bool {
        let launches = self.launches();
        let Some((_, token)) = launches.get(instance_id) else {
            return false;
        };
        token.cancel();
        true
    }

    /// The instances that are being launched.
    pub fn launching(&self) -> Vec<InstanceId> {
        self.launches().keys().cloned().collect()
    }
}

impl RegisteredLaunch {
    /// Takes the launch out of the registry, for the game is about to be started. False when it
    /// was called off before: it must not go on. Decided under the registry's lock, so that a call
    /// off either comes first and is obeyed, or finds no launch and is told so.
    #[must_use = "a launch that was called off must not go on"]
    pub fn commit(self) -> bool {
        let mut launches = self.registry.launches();
        let live = launches
            .get(&self.instance_id)
            .is_some_and(|(key, _)| *key == self.key);
        if live {
            launches.remove(&self.instance_id);
        }
        live && !self.token.is_cancelled()
    }
}

impl Drop for RegisteredLaunch {
    fn drop(&mut self) {
        let mut launches = self.registry.launches();
        if launches
            .get(&self.instance_id)
            .is_some_and(|(key, _)| *key == self.key)
        {
            launches.remove(&self.instance_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(name: &str) -> InstanceId {
        InstanceId::try_from(name).unwrap()
    }

    #[test]
    fn should_fire_the_token_of_the_launch_that_is_called_off_only() {
        let launches = Arc::new(LaunchCancellations::default());
        let mine = launches.register(&id("mine"));
        let other = launches.register(&id("other"));

        assert!(launches.cancel(&id("mine")));

        assert!(mine.token.is_cancelled());
        assert!(!other.token.is_cancelled());
    }

    #[test]
    fn should_find_no_launch_when_none_is_registered_or_it_is_over() {
        let launches = Arc::new(LaunchCancellations::default());
        assert!(!launches.cancel(&id("mine")));

        drop(launches.register(&id("mine")));

        assert!(!launches.cancel(&id("mine")));
    }

    #[test]
    fn should_not_carry_a_call_off_over_to_the_next_launch() {
        let launches = Arc::new(LaunchCancellations::default());
        let first = launches.register(&id("mine"));
        launches.cancel(&id("mine"));
        drop(first);

        let second = launches.register(&id("mine"));

        assert!(!second.token.is_cancelled());
    }

    #[test]
    fn should_keep_the_newer_launch_when_the_older_one_is_dropped_late() {
        let launches = Arc::new(LaunchCancellations::default());
        let older = launches.register(&id("mine"));
        let newer = launches.register(&id("mine"));
        drop(older);

        assert!(launches.cancel(&id("mine")));

        assert!(newer.token.is_cancelled());
    }

    #[test]
    fn should_list_the_instances_being_launched() {
        let launches = Arc::new(LaunchCancellations::default());
        let _mine = launches.register(&id("mine"));
        let other = launches.register(&id("other"));
        drop(other);

        assert_eq!(launches.launching(), vec![id("mine")]);
    }

    #[test]
    fn should_obey_a_call_off_that_comes_before_the_commit() {
        let launches = Arc::new(LaunchCancellations::default());
        let launch = launches.register(&id("mine"));
        launches.cancel(&id("mine"));

        assert!(!launch.commit());
    }

    #[test]
    fn should_find_no_launch_to_call_off_after_the_commit() {
        let launches = Arc::new(LaunchCancellations::default());
        let launch = launches.register(&id("mine"));

        assert!(launch.commit());

        assert!(!launches.cancel(&id("mine")));
        assert!(launches.launching().is_empty());
    }

    #[test]
    fn should_call_off_twice_without_harm() {
        let launches = Arc::new(LaunchCancellations::default());
        let _mine = launches.register(&id("mine"));

        assert!(launches.cancel(&id("mine")));
        assert!(launches.cancel(&id("mine")));
    }
}
