use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::watch;

use super::{ActivityPause, StorageActivity};

#[derive(Clone, Copy, Default)]
struct GateState {
    leases: u32,
    paused: bool,
}

impl GateState {
    /// A pause is only given when there are no leases, so `paused` implies none.
    fn busy(self) -> bool {
        self.leases > 0 || self.paused
    }
}

/// Counts the work that uses the caches, so emptying them waits until it is over.
///
/// Work takes a [`lease`](Self::lease) for as long as it runs; the pause that emptying holds
/// is only given when there are no leases ([`try_pause`](Self::try_pause)). Taking a lease and
/// checking the pause is one change of the state, so nothing slips in between.
///
/// Leases are counted, not exclusive: a lease taken under another never waits (while any is
/// held, the gate cannot be paused). While the gate is paused a new lease waits for it, so
/// nothing may take a lease under a pause.
///
/// Whoever [watches](Self::watch_busy) is woken only when the gate turns busy or idle, not on
/// every lease.
pub struct ActivityGate {
    state: watch::Sender<GateState>,
}

impl Default for ActivityGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivityGate {
    pub fn new() -> Self {
        Self {
            state: watch::Sender::new(GateState::default()),
        }
    }

    /// Wait for the pause to end, if any, and hold the gate busy until the lease is dropped.
    pub async fn lease(self: &Arc<Self>) -> ActivityLease {
        loop {
            if let Some(lease) = self.try_lease() {
                return lease;
            }
            let mut state = self.state.subscribe();
            // The sender is owned by `self`, so it outlives the wait.
            let _ = state.wait_for(|state| !state.paused).await;
        }
    }

    /// A lease, or `None` while the gate is paused.
    pub fn try_lease(self: &Arc<Self>) -> Option<ActivityLease> {
        let mut leased = false;
        self.state.send_if_modified(|state| {
            if state.paused {
                return false;
            }
            leased = true;
            state.leases += 1;
            state.leases == 1
        });
        leased.then(|| ActivityLease { gate: self.clone() })
    }

    /// Keep new leases from being taken until the pause is dropped, or `None` while any is held
    /// or the gate is paused already.
    pub fn try_pause(self: &Arc<Self>) -> Option<ActivityPauseGuard> {
        let mut paused = false;
        self.state.send_if_modified(|state| {
            if state.busy() {
                return false;
            }
            paused = true;
            state.paused = true;
            true
        });
        paused.then(|| ActivityPauseGuard { gate: self.clone() })
    }

    /// Whether anything holds a lease or the gate is paused. Takes nothing.
    pub fn is_busy(&self) -> bool {
        self.state.borrow().busy()
    }

    /// Wakes when [`is_busy`](Self::is_busy) changes.
    pub fn watch_busy(&self) -> BusyWatch {
        BusyWatch {
            state: self.state.subscribe(),
        }
    }

    fn release_lease(&self) {
        self.state.send_if_modified(|state| {
            state.leases -= 1;
            state.leases == 0
        });
    }

    fn release_pause(&self) {
        self.state.send_modify(|state| state.paused = false);
    }
}

/// Holds the gate busy until dropped.
#[must_use = "the lease ends as soon as it is dropped"]
pub struct ActivityLease {
    gate: Arc<ActivityGate>,
}

impl Drop for ActivityLease {
    fn drop(&mut self) {
        self.gate.release_lease();
    }
}

/// Keeps the gate from being leased until dropped.
#[must_use = "the pause ends as soon as it is dropped"]
pub struct ActivityPauseGuard {
    gate: Arc<ActivityGate>,
}

impl Drop for ActivityPauseGuard {
    fn drop(&mut self) {
        self.gate.release_pause();
    }
}

/// Follows [`ActivityGate::is_busy`].
pub struct BusyWatch {
    state: watch::Receiver<GateState>,
}

impl BusyWatch {
    /// Whether the gate is busy now, without waiting.
    pub fn is_busy(&self) -> bool {
        self.state.borrow().busy()
    }

    /// Wait until the gate turns busy or idle. What it turned to, or `None` once the gate is gone.
    pub async fn changed(&mut self) -> Option<bool> {
        self.state.changed().await.ok()?;
        Some(self.state.borrow_and_update().busy())
    }
}

#[async_trait]
impl StorageActivity for Arc<ActivityGate> {
    fn is_busy(&self) -> bool {
        ActivityGate::is_busy(self)
    }

    async fn try_pause(&self) -> Option<ActivityPause> {
        ActivityGate::try_pause(self).map(|pause| Box::new(pause) as ActivityPause)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn gate() -> Arc<ActivityGate> {
        Arc::new(ActivityGate::new())
    }

    #[tokio::test]
    async fn should_refuse_a_pause_when_a_lease_is_held() {
        let gate = gate();
        let lease = gate.lease().await;

        assert!(gate.try_pause().is_none());

        drop(lease);
        assert!(gate.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_refuse_a_second_pause_when_paused() {
        let gate = gate();
        let pause = gate.try_pause().unwrap();

        assert!(gate.try_pause().is_none());

        drop(pause);
        assert!(gate.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_refuse_a_try_lease_when_paused() {
        let gate = gate();
        let pause = gate.try_pause().unwrap();

        assert!(gate.try_lease().is_none());

        drop(pause);
        assert!(gate.try_lease().is_some());
    }

    #[tokio::test]
    async fn should_let_a_lease_wait_when_the_gate_is_paused() {
        let gate = gate();
        let pause = gate.try_pause().unwrap();

        let waiting = tokio::spawn({
            let gate = gate.clone();
            async move { gate.lease().await }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!waiting.is_finished());

        drop(pause);
        let lease = tokio::time::timeout(Duration::from_secs(1), waiting)
            .await
            .unwrap()
            .unwrap();
        assert!(gate.is_busy());
        drop(lease);
        assert!(!gate.is_busy());
    }

    #[tokio::test]
    async fn should_not_make_a_lease_wait_when_taken_under_another() {
        let gate = gate();
        let outer = gate.lease().await;

        let inner = tokio::time::timeout(Duration::from_millis(100), gate.lease())
            .await
            .unwrap();

        drop(outer);
        assert!(gate.is_busy());
        drop(inner);
        assert!(!gate.is_busy());
    }

    #[tokio::test]
    async fn should_leave_the_gate_idle_when_a_waiting_lease_is_cancelled() {
        let gate = gate();
        let pause = gate.try_pause().unwrap();

        let waiting = tokio::time::timeout(Duration::from_millis(20), gate.lease()).await;
        assert!(waiting.is_err());

        drop(pause);
        assert!(!gate.is_busy());
        assert!(gate.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_release_the_lease_when_its_task_panics() {
        let gate = gate();

        let task = tokio::spawn({
            let gate = gate.clone();
            async move {
                let _lease = gate.lease().await;
                panic!("download failed");
            }
        });
        assert!(task.await.is_err());

        assert!(gate.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_wake_a_watcher_only_when_the_gate_turns_busy_or_idle() {
        let gate = gate();
        let mut watch = gate.watch_busy();

        let first = gate.lease().await;
        assert_eq!(watch.changed().await, Some(true));

        let second = gate.lease().await;
        drop(first);
        drop(second);
        assert_eq!(watch.changed().await, Some(false));

        let pause = gate.try_pause().unwrap();
        assert_eq!(watch.changed().await, Some(true));
        drop(pause);
        assert_eq!(watch.changed().await, Some(false));
    }

    #[tokio::test]
    async fn should_not_wake_a_watcher_when_a_lease_is_taken_under_a_lease() {
        let gate = gate();
        let outer = gate.lease().await;
        let mut watch = gate.watch_busy();

        let inner = gate.lease().await;
        drop(inner);

        let woken = tokio::time::timeout(Duration::from_millis(50), watch.changed()).await;
        assert!(woken.is_err());
        drop(outer);
    }

    #[tokio::test]
    async fn should_take_nothing_when_asked_if_busy() {
        let gate = gate();
        assert!(!gate.is_busy());
        let mut watch = gate.watch_busy();

        assert!(!gate.is_busy());
        assert!(!StorageActivity::is_busy(&gate));
        assert!(!watch.is_busy());

        // Taking a pause, even for a moment, would wake the watcher.
        let woken = tokio::time::timeout(Duration::from_millis(50), watch.changed()).await;
        assert!(woken.is_err());
        let _lease = gate.try_lease().unwrap();
        assert!(StorageActivity::is_busy(&gate));
    }
}
