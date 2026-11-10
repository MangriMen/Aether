use std::{
    collections::HashMap,
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::features::instance::InstanceId;

#[derive(Default)]
struct State {
    running: HashMap<InstanceId, Vec<(u64, CancellationToken)>>,
    /// Instances being removed, by how many removals: an install that starts for one is called
    /// off at once, whenever it comes.
    removing: HashMap<InstanceId, usize>,
}

/// The installs of the game that are running, by instance, so that removing an instance calls
/// off the install that would go on downloading for it.
#[derive(Default)]
pub struct InstallCancellations {
    state: Mutex<State>,
    next: AtomicU64,
    /// Told when an install leaves the registry.
    left: Notify,
}

/// An install of the game of an instance, in the registry until dropped.
#[must_use = "the install is out of the registry as soon as this is dropped"]
pub struct RegisteredInstall<'a> {
    registry: &'a InstallCancellations,
    instance_id: InstanceId,
    key: u64,
    /// Fires when the instance is removed.
    pub token: CancellationToken,
}

/// An instance being removed: the installs for it are called off, and new ones are born called
/// off, until this is dropped.
#[must_use = "the instance is no longer being removed as soon as this is dropped"]
pub struct RemovalGuard<'a> {
    registry: &'a InstallCancellations,
    instance_id: InstanceId,
}

impl InstallCancellations {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn register(&self, instance_id: &InstanceId) -> RegisteredInstall<'_> {
        let key = self.next.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();
        let mut state = self.state();
        if state.removing.contains_key(instance_id) {
            token.cancel();
        }
        state
            .running
            .entry(instance_id.clone())
            .or_default()
            .push((key, token.clone()));
        drop(state);
        RegisteredInstall {
            registry: self,
            instance_id: instance_id.clone(),
            key,
            token,
        }
    }

    /// Start the removal of the instance: every install for it is called off, and the ones that
    /// begin meanwhile are too. Waits for them to unwind: they restore the stage of the
    /// instance, which must not race its removal.
    pub async fn begin_removal(&self, instance_id: &InstanceId) -> RemovalGuard<'_> {
        {
            let mut state = self.state();
            *state.removing.entry(instance_id.clone()).or_default() += 1;
            for (_, token) in state.running.get(instance_id).into_iter().flatten() {
                token.cancel();
            }
        }
        let guard = RemovalGuard {
            registry: self,
            instance_id: instance_id.clone(),
        };
        loop {
            // Asked for before the check, so that a leave in between is not missed.
            let left = self.left.notified();
            tokio::pin!(left);
            left.as_mut().enable();
            if !self.state().running.contains_key(instance_id) {
                return guard;
            }
            left.await;
        }
    }
}

impl Drop for RegisteredInstall<'_> {
    fn drop(&mut self) {
        let mut state = self.registry.state();
        if let Some(installs) = state.running.get_mut(&self.instance_id) {
            installs.retain(|(key, _)| *key != self.key);
            if installs.is_empty() {
                state.running.remove(&self.instance_id);
            }
        }
        drop(state);
        self.registry.left.notify_waiters();
    }
}

impl Drop for RemovalGuard<'_> {
    fn drop(&mut self) {
        let mut state = self.registry.state();
        if let Some(count) = state.removing.get_mut(&self.instance_id) {
            *count -= 1;
            if *count == 0 {
                state.removing.remove(&self.instance_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use super::*;

    fn id(name: &str) -> InstanceId {
        InstanceId::try_from(name).unwrap()
    }

    /// Starts the removal of `name` in a task; the install is unwinding while it waits.
    fn begin_removal_of(
        installs: &Arc<InstallCancellations>,
        name: &str,
    ) -> tokio::task::JoinHandle<()> {
        let (installs, id) = (installs.clone(), id(name));
        tokio::spawn(async move {
            let _removal = installs.begin_removal(&id).await;
        })
    }

    #[tokio::test]
    async fn should_fire_the_token_of_the_instance_being_removed_only() {
        let installs = Arc::new(InstallCancellations::default());
        let mine = installs.register(&id("mine"));
        let other = installs.register(&id("other"));

        let _removal = begin_removal_of(&installs, "mine");
        mine.token.cancelled().await;

        assert!(!other.token.is_cancelled());
    }

    #[tokio::test]
    async fn should_fire_every_token_when_an_instance_has_two_installs() {
        let installs = Arc::new(InstallCancellations::default());
        let first = installs.register(&id("mine"));
        let second = installs.register(&id("mine"));

        let _removal = begin_removal_of(&installs, "mine");
        first.token.cancelled().await;
        second.token.cancelled().await;
    }

    #[tokio::test]
    async fn should_wait_for_the_install_to_leave_when_removing() {
        let installs = Arc::new(InstallCancellations::default());
        let install = installs.register(&id("mine"));
        let removal = begin_removal_of(&installs, "mine");
        install.token.cancelled().await;
        tokio::task::yield_now().await;
        assert!(!removal.is_finished(), "the install is still unwinding");

        drop(install);

        tokio::time::timeout(Duration::from_secs(5), removal)
            .await
            .expect("did not stop waiting")
            .unwrap();
    }

    #[tokio::test]
    async fn should_call_off_an_install_born_while_the_instance_is_being_removed() {
        let installs = InstallCancellations::default();
        let removal = installs.begin_removal(&id("mine")).await;

        let born = installs.register(&id("mine"));

        assert!(born.token.is_cancelled());
        drop(born);
        drop(removal);
        assert!(!installs.register(&id("mine")).token.is_cancelled());
    }

    #[tokio::test]
    async fn should_leave_the_other_install_when_one_is_dropped() {
        let installs = Arc::new(InstallCancellations::default());
        let first = installs.register(&id("mine"));
        let second = installs.register(&id("mine"));
        drop(first);

        let _removal = begin_removal_of(&installs, "mine");

        second.token.cancelled().await;
    }

    #[tokio::test]
    async fn should_fire_nothing_when_the_install_is_over() {
        let installs = InstallCancellations::default();
        let finished = installs.register(&id("mine"));
        let token = finished.token.clone();
        drop(finished);

        let _removal = installs.begin_removal(&id("mine")).await;

        assert!(!token.is_cancelled());
    }
}
