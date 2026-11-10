use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::features::instance::InstanceId;

/// The installs of the game that are running, by instance, so that removing an instance calls
/// off the install that would go on downloading for it.
#[derive(Default)]
pub struct InstallCancellations {
    running: Mutex<HashMap<InstanceId, Vec<(u64, CancellationToken)>>>,
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

impl InstallCancellations {
    pub fn register(&self, instance_id: &InstanceId) -> RegisteredInstall<'_> {
        let key = self.next.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();
        self.running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(instance_id.clone())
            .or_default()
            .push((key, token.clone()));
        RegisteredInstall {
            registry: self,
            instance_id: instance_id.clone(),
            key,
            token,
        }
    }

    /// Call off every install running for the instance, and wait for them to unwind: they
    /// restore the stage of the instance, which must not race its removal.
    pub async fn cancel_and_wait(&self, instance_id: &InstanceId) {
        self.cancel(instance_id);
        loop {
            // Asked for before the check, so that a leave in between is not missed.
            let left = self.left.notified();
            tokio::pin!(left);
            left.as_mut().enable();
            if !self.is_running(instance_id) {
                return;
            }
            left.await;
        }
    }

    fn is_running(&self, instance_id: &InstanceId) -> bool {
        self.running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(instance_id)
    }

    /// Call off every install running for the instance.
    pub fn cancel(&self, instance_id: &InstanceId) {
        let running = self
            .running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (_, token) in running.get(instance_id).into_iter().flatten() {
            token.cancel();
        }
    }
}

impl Drop for RegisteredInstall<'_> {
    fn drop(&mut self) {
        let mut running = self
            .registry
            .running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(installs) = running.get_mut(&self.instance_id) {
            installs.retain(|(key, _)| *key != self.key);
            if installs.is_empty() {
                running.remove(&self.instance_id);
            }
        }
        drop(running);
        self.registry.left.notify_waiters();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(name: &str) -> InstanceId {
        InstanceId::try_from(name).unwrap()
    }

    #[test]
    fn should_fire_the_token_when_its_instance_is_cancelled() {
        let installs = InstallCancellations::default();
        let mine = installs.register(&id("mine"));
        let other = installs.register(&id("other"));

        installs.cancel(&id("mine"));

        assert!(mine.token.is_cancelled());
        assert!(!other.token.is_cancelled());
    }

    #[test]
    fn should_fire_every_token_when_an_instance_has_two_installs() {
        let installs = InstallCancellations::default();
        let first = installs.register(&id("mine"));
        let second = installs.register(&id("mine"));

        installs.cancel(&id("mine"));

        assert!(first.token.is_cancelled() && second.token.is_cancelled());
    }

    #[test]
    fn should_leave_the_other_install_when_one_is_dropped() {
        let installs = InstallCancellations::default();
        let first = installs.register(&id("mine"));
        let second = installs.register(&id("mine"));
        drop(first);

        installs.cancel(&id("mine"));

        assert!(second.token.is_cancelled());
    }

    #[tokio::test]
    async fn should_wait_for_the_install_to_leave_when_cancelling_and_waiting() {
        let installs = std::sync::Arc::new(InstallCancellations::default());
        let id = id("mine");
        let install = installs.register(&id);
        let waiting = {
            let (installs, id) = (installs.clone(), id.clone());
            tokio::spawn(async move { installs.cancel_and_wait(&id).await })
        };
        install.token.cancelled().await;
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished(), "the install is still unwinding");

        drop(install);

        tokio::time::timeout(std::time::Duration::from_secs(5), waiting)
            .await
            .expect("did not stop waiting")
            .unwrap();
    }

    #[test]
    fn should_fire_nothing_when_the_install_is_over() {
        let installs = InstallCancellations::default();
        let finished = installs.register(&id("mine"));
        let token = finished.token.clone();
        drop(finished);

        installs.cancel(&id("mine"));

        assert!(!token.is_cancelled());
    }
}
