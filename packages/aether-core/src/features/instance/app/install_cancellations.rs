use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio_util::sync::CancellationToken;

use crate::features::instance::InstanceId;

/// The installs of the game that are running, by instance, so that removing an instance calls
/// off the install that would go on downloading for it.
#[derive(Default)]
pub struct InstallCancellations {
    running: Mutex<HashMap<InstanceId, Vec<(u64, CancellationToken)>>>,
    next: AtomicU64,
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
