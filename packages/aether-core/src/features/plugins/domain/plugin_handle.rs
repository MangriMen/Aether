use std::{
    cell::RefCell,
    sync::{Arc, Mutex as SyncMutex, MutexGuard as SyncMutexGuard, PoisonError},
};

use tokio::sync::{Mutex, OwnedMutexGuard};

use super::PluginInstance;
use crate::features::{
    instance::PluginId,
    plugins::{FolderMounts, PluginCallError, PluginError},
};

thread_local! {
    /// The plugins whose calls this thread is inside of, outermost first. Everything a host
    /// function waits on is driven on the thread of the call it serves, so a chain of calls that
    /// comes back to one of these plugins shows up here.
    static CALL_CHAIN: RefCell<Vec<PluginId>> = const { RefCell::new(Vec::new()) };
}

/// A loaded plugin, shared by everything that calls into it.
///
/// A call into a plugin is synchronous and may run for minutes, so it is made only through
/// [`PluginHandle::run`], on tokio's blocking pool: on a runtime worker it would stall every
/// task there. Calls into one instance take turns; a caller waits without holding a thread.
///
/// Unloading or restarting retires the handle ([`PluginHandle::retire`]), and every clone still
/// held refuses further calls with [`PluginError::Stopped`]. A restarted plugin gets a new
/// handle, so nothing keeps calling it with the rights it had before.
#[derive(Clone)]
pub struct PluginHandle {
    id: PluginId,
    instance: Arc<Mutex<Option<Box<dyn PluginInstance>>>>,
    calls: Arc<SyncMutex<Calls>>,
    folders: Arc<FolderMounts>,
    saved_folders: Arc<FolderMounts>,
}

/// One lock for both, so that no call can start between the check that none is under way and
/// the retirement.
#[derive(Default)]
struct Calls {
    /// Calls started and not over yet, running or still waiting for their turn.
    started: usize,
    retired: bool,
}

/// Counts one call in [`Calls::started`] for as long as it lives.
struct CountedCall(Arc<SyncMutex<Calls>>);

impl Drop for CountedCall {
    fn drop(&mut self) {
        lock(&self.0).started -= 1;
    }
}

/// Nothing panics while the lock is held, so a poisoned lock still holds the right counts.
fn lock(calls: &SyncMutex<Calls>) -> SyncMutexGuard<'_, Calls> {
    calls.lock().unwrap_or_else(PoisonError::into_inner)
}

impl PluginHandle {
    pub fn new(instance: impl PluginInstance + 'static) -> Self {
        Self {
            id: instance.get_id(),
            instance: Arc::new(Mutex::new(Some(Box::new(instance)))),
            calls: Arc::default(),
            folders: Arc::default(),
            saved_folders: Arc::default(),
        }
    }

    /// Records the user's folders the instance was built with: those mounted, which are the
    /// saved ones less those the loader refused.
    #[must_use]
    pub fn with_folders(mut self, folders: FolderMounts) -> Self {
        self.folders = Arc::new(folders);
        self
    }

    /// Records the folders the settings held when the instance was loaded, whether or not they
    /// were mounted: what a change of the settings is told from.
    #[must_use]
    pub fn with_saved_folders(mut self, folders: FolderMounts) -> Self {
        self.saved_folders = Arc::new(folders);
        self
    }

    /// Known without waiting for a call in progress.
    pub fn id(&self) -> &PluginId {
        &self.id
    }

    /// The user's folders mounted into this instance. Folders are fixed when an instance is
    /// built, so settings saved since take effect only with a new instance.
    pub fn folders(&self) -> &FolderMounts {
        &self.folders
    }

    /// The folders the settings held when this instance was loaded: unlike
    /// [`PluginHandle::folders`], also those the loader left out, so that comparing with the
    /// settings as saved tells whether they changed since.
    pub fn saved_folders(&self) -> &FolderMounts {
        &self.saved_folders
    }

    /// Whether a call is running or waiting for its turn, so that [`PluginHandle::retire`] would
    /// be refused now.
    pub fn is_busy(&self) -> bool {
        lock(&self.calls).started > 0
    }

    pub fn is_retired(&self) -> bool {
        lock(&self.calls).retired
    }

    /// Runs `f` on the blocking pool with the plugin to itself; `function_name` is what `f`
    /// calls, for the error.
    ///
    /// A call that `f` itself leads to (through host functions, directly or via other plugins)
    /// would wait for its own turn forever, so it is refused with [`PluginCallError::Reentrant`].
    /// A chain that leaves the call's thread, such as a spawned task, still deadlocks.
    ///
    /// Once started, the call runs to the end even if the caller stops waiting, so the plugin is
    /// never left halfway. A retired handle refuses with [`PluginError::Stopped`].
    pub async fn run<T, F>(&self, function_name: &str, f: F) -> Result<T, PluginError>
    where
        F: FnOnce(&mut dyn PluginInstance) -> T + Send + 'static,
        T: Send + 'static,
    {
        let mut chain = CALL_CHAIN.with_borrow(Clone::clone);
        if chain.contains(&self.id) {
            return Err(PluginError::FunctionCallFailed {
                function_name: function_name.to_owned(),
                plugin_id: self.id.clone(),
                error: PluginCallError::Reentrant,
            });
        }
        chain.push(self.id.clone());

        // Counted from before the wait for the turn: a caller in line is an operation under way.
        let counted = self.count_call()?;
        let slot = Arc::clone(&self.instance).lock_owned().await;
        // Only a retirement empties the slot, and none happens while a call is counted.
        let Ok(mut instance) = OwnedMutexGuard::try_map(slot, Option::as_deref_mut) else {
            return Err(self.stopped());
        };

        Ok(on_blocking_pool(&self.id, move || {
            let _counted = counted;
            let _chain = ChainScope::enter(chain);
            f(&mut *instance)
        })
        .await)
    }

    /// Ends the plugin's instance: `f` is its last call, and the instance is dropped right after
    /// it, with the folders and rights it was built with.
    ///
    /// Refused with [`PluginError::Busy`] while any call is running or waiting, rather than
    /// waiting for it: a call may take minutes. Refused with [`PluginError::AlreadyUnloaded`]
    /// for a handle already retired.
    pub async fn retire<T, F>(&self, f: F) -> Result<T, PluginError>
    where
        F: FnOnce(&mut dyn PluginInstance) -> T + Send + 'static,
        T: Send + 'static,
    {
        {
            let mut calls = lock(&self.calls);
            if calls.retired {
                return Err(PluginError::AlreadyUnloaded {
                    plugin_id: self.id.clone(),
                });
            }
            if calls.started > 0 {
                return Err(PluginError::Busy {
                    plugin_id: self.id.clone(),
                });
            }
            calls.retired = true;
        }

        // No call is counted and none can start any more: the turn is free, or about to be
        // given back by a call that has just stopped counting.
        let Some(mut instance) = self.instance.lock().await.take() else {
            return Err(PluginError::AlreadyUnloaded {
                plugin_id: self.id.clone(),
            });
        };
        let mut chain = CALL_CHAIN.with_borrow(Clone::clone);
        chain.push(self.id.clone());

        Ok(on_blocking_pool(&self.id, move || {
            let _chain = ChainScope::enter(chain);
            f(&mut *instance)
        })
        .await)
    }

    fn count_call(&self) -> Result<CountedCall, PluginError> {
        let mut calls = lock(&self.calls);
        if calls.retired {
            return Err(self.stopped());
        }
        calls.started += 1;
        Ok(CountedCall(Arc::clone(&self.calls)))
    }

    fn stopped(&self) -> PluginError {
        PluginError::Stopped {
            plugin_id: self.id.clone(),
        }
    }
}

async fn on_blocking_pool<T: Send + 'static>(
    plugin_id: &PluginId,
    call: impl FnOnce() -> T + Send + 'static,
) -> T {
    match tokio::task::spawn_blocking(call).await {
        Ok(result) => result,
        Err(err) if err.is_panic() => std::panic::resume_unwind(err.into_panic()),
        // Only a runtime shutting down drops a blocking task before it runs.
        Err(err) => panic!("plugin '{plugin_id}' call was not run: {err}"),
    }
}

/// The call chain of a blocking-pool thread for as long as it runs one call. The thread goes
/// back to the pool afterwards, so the previous chain is put back even when the call panics.
struct ChainScope(Vec<PluginId>);

impl ChainScope {
    fn enter(chain: Vec<PluginId>) -> Self {
        Self(CALL_CHAIN.replace(chain))
    }
}

impl Drop for ChainScope {
    fn drop(&mut self) {
        CALL_CHAIN.set(std::mem::take(&mut self.0));
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        panic::AssertUnwindSafe,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        time::Duration,
    };

    use futures::FutureExt;
    use tokio::runtime::Handle;

    use super::*;
    use crate::features::plugins::PluginInternalEvent;

    struct Stub(&'static str);

    impl PluginInstance for Stub {
        fn get_id(&self) -> PluginId {
            PluginId::from_static(self.0)
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

    const PATIENCE: Duration = Duration::from_secs(5);

    /// A call blocked on the runtime's own thread would never be released by the test, so the
    /// blocked call is let go after a while anyway: a regression fails the test, not hangs it.
    fn release_later(release: &mpsc::Sender<()>) {
        let release = release.clone();
        std::thread::spawn(move || {
            std::thread::sleep(PATIENCE);
            let _ = release.send(());
        });
    }

    /// Runs `future` on a runtime of its own, which is let go rather than waited for if the
    /// future does not finish in time: a deadlocked call would hang the test in the runtime's
    /// drop otherwise.
    fn within_patience<F: Future>(future: F) -> Option<F::Output> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let output = runtime.block_on(async { tokio::time::timeout(PATIENCE, future).await.ok() });
        runtime.shutdown_background();
        output
    }

    fn is_reentrant(result: &Result<(), PluginError>, plugin: &str, function: &str) -> bool {
        matches!(
            result,
            Err(PluginError::FunctionCallFailed {
                function_name,
                plugin_id,
                error: PluginCallError::Reentrant,
            }) if plugin_id.as_str() == plugin && function_name == function
        )
    }

    /// One worker only: a call made on it would leave nothing to run the other task.
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_hold_up_no_other_task_when_a_call_is_in_progress() {
        let handle = PluginHandle::new(Stub("stub"));
        let (release, released) = mpsc::channel::<()>();
        release_later(&release);
        let (started, call_started) = tokio::sync::oneshot::channel();

        let call = tokio::spawn({
            let handle = handle.clone();
            async move {
                handle
                    .run("wait", move |_| {
                        started.send(()).unwrap();
                        released.recv().is_ok()
                    })
                    .await
            }
        });
        call_started.await.unwrap();

        assert_eq!(tokio::spawn(async { 42 }).await.unwrap(), 42);
        assert!(
            !call.is_finished(),
            "the other task ran while the call was in progress"
        );

        release.send(()).unwrap();
        assert!(call.await.unwrap().unwrap());
    }

    #[tokio::test]
    async fn should_take_turns_when_calls_go_into_one_plugin() {
        let handle = PluginHandle::new(Stub("stub"));
        let (release, released) = mpsc::channel::<()>();
        release_later(&release);
        let (started, first_started) = tokio::sync::oneshot::channel();
        let first_done = Arc::new(AtomicBool::new(false));

        let first = tokio::spawn({
            let handle = handle.clone();
            let first_done = Arc::clone(&first_done);
            async move {
                handle
                    .run("first", move |_| {
                        started.send(()).unwrap();
                        released.recv().unwrap();
                        first_done.store(true, Ordering::SeqCst);
                        "first"
                    })
                    .await
            }
        });
        first_started.await.unwrap();

        let second = tokio::spawn({
            let handle = handle.clone();
            let first_done = Arc::clone(&first_done);
            async move {
                handle
                    .run("second", move |_| first_done.load(Ordering::SeqCst))
                    .await
            }
        });

        assert_eq!(
            handle.id().as_str(),
            "stub",
            "the id is there during a call"
        );
        for _ in 0..16 {
            tokio::task::yield_now().await;
        }
        assert!(!second.is_finished(), "the second call waits for the first");

        release.send(()).unwrap();
        assert_eq!(first.await.unwrap().unwrap(), "first");
        assert!(
            second.await.unwrap().unwrap(),
            "the second call ran before the first one finished"
        );
    }

    #[tokio::test]
    async fn should_reach_the_caller_and_stay_usable_when_a_call_panics() {
        let handle = PluginHandle::new(Stub("stub"));

        let panicked = AssertUnwindSafe(handle.run("boom", |_| -> () { panic!("boom") }))
            .catch_unwind()
            .await;
        assert!(panicked.is_err());

        let after = tokio::time::timeout(PATIENCE, handle.run("id", |plugin| plugin.get_id()))
            .await
            .expect("the plugin should not stay taken");
        assert_eq!(after.unwrap(), PluginId::from_static("stub"));
    }

    /// What a host function does: blocks the call's thread on async work, here a call back
    /// into the same plugin.
    #[test]
    fn should_refuse_a_call_back_into_the_plugin_when_it_is_already_running() {
        let plugin = PluginHandle::new(Stub("a"));
        let again = plugin.clone();

        let result = within_patience(async move {
            plugin
                .run("outer", move |_| {
                    Handle::current().block_on(again.run("inner", |_| ()))
                })
                .await
        })
        .expect("the call back should not deadlock");

        assert!(is_reentrant(&result.unwrap(), "a", "inner"));
    }

    #[test]
    fn should_refuse_a_call_back_through_another_plugin_when_it_leads_to_the_running_one() {
        let a = PluginHandle::new(Stub("a"));
        let b = PluginHandle::new(Stub("b"));
        let a_again = a.clone();

        let result = within_patience(async move {
            a.run("to_b", move |_| {
                Handle::current().block_on(b.run("to_a", move |_| {
                    Handle::current().block_on(a_again.run("back", |_| ()))
                }))
            })
            .await
        })
        .expect("the call back should not deadlock");

        let through_b = result
            .unwrap()
            .expect("a call into another plugin is allowed");
        assert!(is_reentrant(&through_b, "a", "back"));
    }

    /// Blocking-pool threads are reused: a chain left behind would refuse unrelated calls.
    #[test]
    fn should_end_the_call_chain_when_the_call_panics() {
        let panicked = std::panic::catch_unwind(|| {
            let _chain = ChainScope::enter(vec![PluginId::from_static("a")]);
            panic!("boom");
        });

        assert!(panicked.is_err());
        assert!(CALL_CHAIN.with_borrow(Vec::is_empty));
    }

    /// Tells the test when the instance is dropped.
    struct Dropped(Arc<std::sync::atomic::AtomicBool>);

    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    struct Tracked {
        _dropped: Dropped,
    }

    impl PluginInstance for Tracked {
        fn get_id(&self) -> PluginId {
            PluginId::from_static("tracked")
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

    fn is_busy_error<T>(result: &Result<T, PluginError>) -> bool {
        matches!(result, Err(PluginError::Busy { plugin_id }) if plugin_id.as_str() == "stub")
    }

    #[tokio::test]
    async fn should_end_the_instance_after_its_last_call_when_it_is_retired() {
        let dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = PluginHandle::new(Tracked {
            _dropped: Dropped(Arc::clone(&dropped)),
        });
        let clone = handle.clone();

        let last = handle.retire(|plugin| plugin.get_id()).await.unwrap();

        assert_eq!(last, PluginId::from_static("tracked"));
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
        assert!(clone.is_retired());
        assert!(matches!(
            clone.run("after", |_| ()).await,
            Err(PluginError::Stopped { plugin_id }) if plugin_id == PluginId::from_static("tracked")
        ));
        assert!(matches!(
            clone.retire(|_| ()).await,
            Err(PluginError::AlreadyUnloaded { .. })
        ));
    }

    #[tokio::test]
    async fn should_refuse_a_retirement_when_a_call_is_running() {
        let handle = PluginHandle::new(Stub("stub"));
        let (release, released) = mpsc::channel::<()>();
        release_later(&release);
        let (started, call_started) = tokio::sync::oneshot::channel();

        let call = tokio::spawn({
            let handle = handle.clone();
            async move {
                handle
                    .run("long", move |plugin| {
                        started.send(()).unwrap();
                        released.recv().unwrap();
                        plugin.get_id()
                    })
                    .await
            }
        });
        call_started.await.unwrap();

        assert!(handle.is_busy());
        assert!(is_busy_error(&handle.retire(|_| ()).await));
        assert!(!handle.is_retired());

        release.send(()).unwrap();
        assert_eq!(call.await.unwrap().unwrap(), PluginId::from_static("stub"));
        assert!(handle.retire(|_| ()).await.is_ok());
    }

    #[tokio::test]
    async fn should_not_count_a_caller_when_it_stops_waiting_for_its_turn() {
        let handle = PluginHandle::new(Stub("stub"));
        let (release, released) = mpsc::channel::<()>();
        release_later(&release);
        let (started, first_started) = tokio::sync::oneshot::channel();

        let first = tokio::spawn({
            let handle = handle.clone();
            async move {
                handle
                    .run("first", move |_| {
                        started.send(()).unwrap();
                        released.recv().unwrap();
                    })
                    .await
            }
        });
        first_started.await.unwrap();

        let gave_up =
            tokio::time::timeout(Duration::from_millis(50), handle.run("second", |_| ())).await;
        assert!(gave_up.is_err(), "the second call was still waiting");

        release.send(()).unwrap();
        first.await.unwrap().unwrap();
        assert!(!handle.is_busy());
        assert!(handle.retire(|_| ()).await.is_ok());
    }
}
