//! The questions "may this plugin run this command?" that wait for the user.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::features::events::{
    CommandApprovalOutcome, EventEmitterExt, PluginEvent, SharedEventEmitter,
};
use crate::features::instance::PluginId;
use crate::features::plugins::{
    CommandApprovalAnswer, CommandApprovalRefusal, CommandApprovalRequest, CommandGrant,
    PendingCommandApproval, PluginError,
};

use super::ports::CommandApprover;

/// How long a question waits before it is refused.
pub const COMMAND_APPROVAL_TIMEOUT: Duration = Duration::from_mins(5);

struct Waiting {
    pending: PendingCommandApproval,
    /// Taken when the question is answered or cancelled; dropping it wakes the waiter.
    answer: Option<oneshot::Sender<CommandApprovalAnswer>>,
}

#[derive(Default)]
struct State {
    waiting: HashMap<Uuid, Waiting>,
    /// Commands the user denied, per plugin; refused again until the plugin is stopped.
    denied: HashMap<PluginId, HashSet<DenialKey>>,
}

pub struct PendingCommandApprovals {
    events: SharedEventEmitter,
    timeout: Duration,
    state: Mutex<State>,
}

impl PendingCommandApprovals {
    pub fn new(events: SharedEventEmitter) -> Self {
        Self::with_timeout(events, COMMAND_APPROVAL_TIMEOUT)
    }

    pub fn with_timeout(events: SharedEventEmitter, timeout: Duration) -> Self {
        Self {
            events,
            timeout,
            state: Mutex::new(State::default()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The command as the user saw it: the program, its folder and its arguments.
#[derive(PartialEq, Eq, Hash)]
struct DenialKey {
    java: String,
    current_dir: String,
    arguments: Vec<String>,
}

impl DenialKey {
    fn of(request: &CommandApprovalRequest) -> Self {
        Self {
            java: request.java.clone(),
            current_dir: request.current_dir.clone(),
            arguments: request.arguments.clone(),
        }
    }
}

/// Removes the question from the map however the wait ends, the future being dropped included.
struct WaitGuard<'a> {
    approvals: &'a PendingCommandApprovals,
    id: Uuid,
}

impl Drop for WaitGuard<'_> {
    fn drop(&mut self) {
        // The normal end removes the question before it reports; one still here means the wait
        // was dropped without ending, so the UI is told the question is gone.
        let Some(left) = self.approvals.lock().waiting.remove(&self.id) else {
            return;
        };
        let events = self.approvals.events.clone();
        let (plugin_id, request_id) = (left.pending.request.plugin_id, self.id);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                events
                    .emit_safe(PluginEvent::CommandApprovalResolved {
                        plugin_id,
                        request_id,
                        outcome: CommandApprovalOutcome::Cancelled,
                    })
                    .await;
            });
        }
    }
}

#[async_trait]
impl CommandApprover for PendingCommandApprovals {
    async fn request(
        &self,
        request: CommandApprovalRequest,
    ) -> Result<CommandGrant, CommandApprovalRefusal> {
        let key = DenialKey::of(&request);
        let id = request.id;
        let plugin_id = request.plugin_id.clone();
        let (sender, receiver) = oneshot::channel();

        {
            let mut state = self.lock();
            if state
                .denied
                .get(&request.plugin_id)
                .is_some_and(|denied| denied.contains(&key))
            {
                return Err(CommandApprovalRefusal::Denied);
            }
            let expires_at: DateTime<Utc> = Utc::now()
                + chrono::Duration::from_std(self.timeout).unwrap_or(chrono::Duration::zero());
            state.waiting.insert(
                id,
                Waiting {
                    pending: PendingCommandApproval {
                        request,
                        expires_at,
                    },
                    answer: Some(sender),
                },
            );
        }
        let _guard = WaitGuard {
            approvals: self,
            id,
        };

        self.events
            .emit_safe(PluginEvent::CommandApprovalRequested {
                plugin_id: plugin_id.clone(),
                request_id: id,
            })
            .await;

        let outcome = tokio::time::timeout(self.timeout, receiver).await;

        self.lock().waiting.remove(&id);
        let ended = match &outcome {
            Err(_) => CommandApprovalOutcome::TimedOut,
            // The sender went away without an answer: the plugin was cancelled.
            Ok(Err(_)) => CommandApprovalOutcome::Cancelled,
            Ok(Ok(_)) => CommandApprovalOutcome::Answered,
        };
        self.events
            .emit_safe(PluginEvent::CommandApprovalResolved {
                plugin_id,
                request_id: id,
                outcome: ended,
            })
            .await;

        match outcome {
            Err(_) => Err(CommandApprovalRefusal::TimedOut),
            // The sender went away without an answer: the plugin was cancelled.
            Ok(Err(_)) => Err(CommandApprovalRefusal::Cancelled),
            Ok(Ok(CommandApprovalAnswer::Once)) => Ok(CommandGrant::Once),
            Ok(Ok(CommandApprovalAnswer::Always)) => Ok(CommandGrant::Always),
            Ok(Ok(CommandApprovalAnswer::Deny)) => Err(CommandApprovalRefusal::Denied),
        }
    }

    fn pending(&self) -> Vec<PendingCommandApproval> {
        let state = self.lock();
        let mut pending: Vec<_> = state.waiting.values().map(|w| w.pending.clone()).collect();
        pending.sort_by_key(|p| p.expires_at);
        pending
    }

    fn answer(&self, request_id: Uuid, answer: CommandApprovalAnswer) -> Result<(), PluginError> {
        let not_found = || PluginError::CommandApprovalNotFound {
            request_id: request_id.to_string(),
        };
        let mut state = self.lock();
        let waiting = state.waiting.get_mut(&request_id).ok_or_else(not_found)?;
        if answer == CommandApprovalAnswer::Always && waiting.pending.request.always.is_err() {
            return Err(PluginError::CommandApprovalOnceOnly {
                request_id: request_id.to_string(),
            });
        }
        let sender = waiting.answer.take().ok_or_else(not_found)?;
        let key = DenialKey::of(&waiting.pending.request);
        let plugin_id = waiting.pending.request.plugin_id.clone();
        // The waiter may have just given up (its timeout fired before it removed the entry): the
        // answer reached nobody, which the caller is told.
        sender.send(answer).map_err(|_| not_found())?;
        if answer == CommandApprovalAnswer::Deny {
            // Recorded here, under the lock a stop takes too, so a stop that follows cannot be
            // outlived by a denial the waiter writes after waking.
            state.denied.entry(plugin_id).or_default().insert(key);
        }
        Ok(())
    }

    fn forget_denials(&self, plugin_id: &PluginId) {
        self.lock().denied.remove(plugin_id);
    }

    fn cancel_plugin(&self, plugin_id: &PluginId) -> bool {
        let mut state = self.lock();
        let mut cancelled = false;
        for waiting in state.waiting.values_mut() {
            if waiting.pending.request.plugin_id == *plugin_id {
                // Dropping the sender wakes the waiter with `Cancelled`.
                cancelled |= waiting.answer.take().is_some();
            }
        }
        cancelled
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::features::events::{CommandApprovalOutcome, Event, EventEmitter, EventError};
    use crate::features::plugins::{CommandLine, OnceOnlyReason};

    struct NullEmitter;

    #[async_trait]
    impl EventEmitter<Event> for NullEmitter {
        async fn emit(&self, _event: Event) -> Result<(), EventError> {
            Ok(())
        }
        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    fn approvals(timeout: Duration) -> Arc<PendingCommandApprovals> {
        Arc::new(PendingCommandApprovals::with_timeout(
            Arc::new(NullEmitter),
            timeout,
        ))
    }

    fn request(plugin: &str, once_only: bool) -> CommandApprovalRequest {
        CommandApprovalRequest {
            id: Uuid::new_v4(),
            plugin_id: PluginId::try_from(plugin).unwrap(),
            java: "java".to_owned(),
            current_dir: "/cache".to_owned(),
            arguments: vec!["-jar".to_owned(), "#/cache/a.jar".to_owned()],
            line: CommandLine::default(),
            jar: None,
            always: if once_only {
                Err(OnceOnlyReason::ArgFile)
            } else {
                Ok(())
            },
        }
    }

    fn spawn_request(
        approvals: &Arc<PendingCommandApprovals>,
        request: CommandApprovalRequest,
    ) -> tokio::task::JoinHandle<Result<CommandGrant, CommandApprovalRefusal>> {
        let approvals = approvals.clone();
        tokio::spawn(async move { approvals.request(request).await })
    }

    async fn wait_for_pending(approvals: &PendingCommandApprovals) -> Uuid {
        loop {
            if let Some(p) = approvals.pending().first() {
                return p.request.id;
            }
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test]
    async fn should_grant_when_the_user_answers_always() {
        let approvals = approvals(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", false));

        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Always).unwrap();

        assert_eq!(waiter.await.unwrap(), Ok(CommandGrant::Always));
        assert!(approvals.pending().is_empty());
    }

    #[tokio::test]
    async fn should_refuse_always_and_keep_waiting_when_only_once_is_allowed() {
        let approvals = approvals(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", true));

        let id = wait_for_pending(&approvals).await;
        let result = approvals.answer(id, CommandApprovalAnswer::Always);

        assert!(matches!(
            result,
            Err(PluginError::CommandApprovalOnceOnly { .. })
        ));
        assert_eq!(approvals.pending().len(), 1);
        approvals.answer(id, CommandApprovalAnswer::Once).unwrap();
        assert_eq!(waiter.await.unwrap(), Ok(CommandGrant::Once));
    }

    #[tokio::test]
    async fn should_not_ask_again_when_the_same_command_was_denied() {
        let approvals = approvals(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", false));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Deny).unwrap();
        assert_eq!(waiter.await.unwrap(), Err(CommandApprovalRefusal::Denied));

        let again = approvals.request(request("p", false)).await;

        assert_eq!(again, Err(CommandApprovalRefusal::Denied));
        assert!(approvals.pending().is_empty());
    }

    #[tokio::test]
    async fn should_ask_again_when_the_plugin_was_stopped_after_a_denial() {
        let approvals = approvals(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", false));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Deny).unwrap();
        let _ = waiter.await;

        approvals.forget_denials(&PluginId::try_from("p").unwrap());
        let waiter = spawn_request(&approvals, request("p", false));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Once).unwrap();

        assert_eq!(waiter.await.unwrap(), Ok(CommandGrant::Once));
    }

    #[tokio::test]
    async fn should_cancel_only_the_questions_of_that_plugin() {
        let approvals = approvals(Duration::from_secs(60));
        let a = spawn_request(&approvals, request("a", false));
        let b = spawn_request(&approvals, request("b", false));
        while approvals.pending().len() < 2 {
            tokio::task::yield_now().await;
        }

        approvals.cancel_plugin(&PluginId::try_from("a").unwrap());

        assert_eq!(a.await.unwrap(), Err(CommandApprovalRefusal::Cancelled));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Once).unwrap();
        assert_eq!(b.await.unwrap(), Ok(CommandGrant::Once));
    }

    #[tokio::test]
    async fn should_time_out_when_nobody_answers() {
        let approvals = approvals(Duration::from_millis(30));

        let result = approvals.request(request("p", false)).await;

        assert_eq!(result, Err(CommandApprovalRefusal::TimedOut));
        assert!(approvals.pending().is_empty());
    }

    #[tokio::test]
    async fn should_report_not_found_when_the_question_is_gone() {
        let approvals = approvals(Duration::from_secs(60));

        let result = approvals.answer(Uuid::new_v4(), CommandApprovalAnswer::Once);

        assert!(matches!(
            result,
            Err(PluginError::CommandApprovalNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn should_keep_a_denial_when_the_plugin_was_only_asked_to_cancel() {
        let approvals = approvals(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", false));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Deny).unwrap();
        let _ = waiter.await;

        // A stop that was refused as busy cancels questions but forgets nothing.
        approvals.cancel_plugin(&PluginId::try_from("p").unwrap());

        let again = approvals.request(request("p", false)).await;
        assert_eq!(again, Err(CommandApprovalRefusal::Denied));
    }
    /// The approvals, and what they say about how each question ended.
    fn approvals_with_outcomes(
        timeout: Duration,
    ) -> (
        Arc<PendingCommandApprovals>,
        tokio::sync::mpsc::UnboundedReceiver<Event>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let approvals = Arc::new(PendingCommandApprovals::with_timeout(
            Arc::new(crate::features::events::test_support::ChannelEmitter(
                sender,
            )),
            timeout,
        ));
        (approvals, receiver)
    }

    fn resolved(
        receiver: &mut tokio::sync::mpsc::UnboundedReceiver<Event>,
    ) -> Vec<CommandApprovalOutcome> {
        let mut outcomes = vec![];
        while let Ok(event) = receiver.try_recv() {
            if let Event::Plugin(PluginEvent::CommandApprovalResolved { outcome, .. }) = event {
                outcomes.push(outcome);
            }
        }
        outcomes
    }

    #[tokio::test]
    async fn should_report_a_timeout_when_nobody_answers() {
        let (approvals, mut events) = approvals_with_outcomes(Duration::from_millis(30));

        let _ = approvals.request(request("p", false)).await;

        assert_eq!(
            resolved(&mut events),
            vec![CommandApprovalOutcome::TimedOut]
        );
    }

    #[tokio::test]
    async fn should_report_a_cancel_when_the_plugin_is_stopped() {
        let (approvals, mut events) = approvals_with_outcomes(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", false));
        wait_for_pending(&approvals).await;

        approvals.cancel_plugin(&PluginId::try_from("p").unwrap());
        let _ = waiter.await;

        assert_eq!(
            resolved(&mut events),
            vec![CommandApprovalOutcome::Cancelled]
        );
    }

    #[tokio::test]
    async fn should_report_answered_when_the_user_allows_or_denies() {
        let (approvals, mut events) = approvals_with_outcomes(Duration::from_secs(60));
        let first = spawn_request(&approvals, request("p", false));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Once).unwrap();
        let _ = first.await;

        let mut other = request("q", false);
        other.arguments.push("x".into());
        let second = spawn_request(&approvals, other);
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Deny).unwrap();
        let _ = second.await;

        assert_eq!(
            resolved(&mut events),
            vec![
                CommandApprovalOutcome::Answered,
                CommandApprovalOutcome::Answered
            ]
        );
    }

    #[tokio::test]
    async fn should_say_nothing_when_a_remembered_denial_refuses_the_command() {
        let (approvals, mut events) = approvals_with_outcomes(Duration::from_secs(60));
        let first = spawn_request(&approvals, request("p", false));
        let id = wait_for_pending(&approvals).await;
        approvals.answer(id, CommandApprovalAnswer::Deny).unwrap();
        let _ = first.await;
        let before = resolved(&mut events).len();

        let again = approvals.request(request("p", false)).await;

        assert_eq!(again, Err(CommandApprovalRefusal::Denied));
        assert_eq!(before, 1);
        assert!(
            events.try_recv().is_err(),
            "no event for a refusal nobody was asked about"
        );
    }

    #[tokio::test]
    async fn should_report_a_cancel_when_the_wait_is_dropped_before_it_ends() {
        let (approvals, mut events) = approvals_with_outcomes(Duration::from_secs(60));
        let waiter = spawn_request(&approvals, request("p", false));
        wait_for_pending(&approvals).await;

        waiter.abort();
        let _ = waiter.await;

        let resolved = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                match events.recv().await {
                    Some(Event::Plugin(PluginEvent::CommandApprovalResolved {
                        outcome, ..
                    })) => return outcome,
                    Some(_) => {}
                    None => panic!("the channel closed"),
                }
            }
        })
        .await
        .expect("the UI was never told the question is gone");
        assert_eq!(resolved, CommandApprovalOutcome::Cancelled);
        assert!(approvals.pending().is_empty());
    }
}
