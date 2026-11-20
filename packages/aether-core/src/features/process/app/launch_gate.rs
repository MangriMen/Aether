use std::sync::Arc;

use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

/// Keeps a restart of the launcher from coming between the start of a game and the saving of what
/// a later launcher needs to follow it: launches share the gate, a restart takes it alone and
/// holds it until the launcher is gone.
#[derive(Default)]
pub struct LaunchGate {
    gate: Arc<RwLock<()>>,
}

/// A launch in the gate, until dropped.
#[must_use = "the launch is out of the gate as soon as this is dropped"]
pub struct LaunchPermit {
    _guard: OwnedRwLockReadGuard<()>,
}

/// The gate held by a restart, until dropped: no game is launched under it.
#[must_use = "the gate is open again as soon as this is dropped"]
pub struct RestartPermit {
    _guard: OwnedRwLockWriteGuard<()>,
}

impl RestartPermit {
    /// Keeps the gate closed for good, as the launcher is going away: a launch that comes now
    /// would be left to a launcher that is not there.
    pub fn keep_closed(self) {
        std::mem::forget(self);
    }
}

impl LaunchGate {
    /// Enters the gate for a launch. Waits while a restart holds it, which ends with the launcher.
    pub async fn enter_launch(&self) -> LaunchPermit {
        LaunchPermit {
            _guard: self.gate.clone().read_owned().await,
        }
    }

    /// Takes the gate for a restart. `None` while a launch is in it.
    pub fn try_restart(&self) -> Option<RestartPermit> {
        self.gate
            .clone()
            .try_write_owned()
            .ok()
            .map(|guard| RestartPermit { _guard: guard })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_refuse_a_restart_while_a_launch_is_in_the_gate() {
        let gate = LaunchGate::default();
        let launch = gate.enter_launch().await;

        assert!(gate.try_restart().is_none());

        drop(launch);
        assert!(gate.try_restart().is_some());
    }

    #[tokio::test]
    async fn should_let_launches_in_together() {
        let gate = LaunchGate::default();
        let _first = gate.enter_launch().await;

        let second = tokio::time::timeout(Duration::from_secs(1), gate.enter_launch()).await;

        assert!(second.is_ok());
    }

    #[tokio::test]
    async fn should_hold_a_launch_back_while_a_restart_holds_the_gate() {
        let gate = Arc::new(LaunchGate::default());
        let restart = gate.try_restart().unwrap();

        let waiting = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.enter_launch().await })
        };
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());

        drop(restart);
        assert!(waiting.await.is_ok());
    }
}
