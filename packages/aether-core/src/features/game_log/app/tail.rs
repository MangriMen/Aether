use std::time::Duration;

use tokio::sync::oneshot;

use crate::features::instance::InstanceId;

/// Follows the lines a running game appends to its own `latest.log` and announces them as they
/// come, for a game whose console output the launcher does not capture.
pub trait GameLogTailService: Send + Sync {
    /// Follows the file from its end now, until the handle is finished or dropped.
    fn follow(&self, instance_id: &InstanceId) -> LogTail;
}

/// A following in progress. Dropping it stops the following after one last look at the file.
pub struct LogTail {
    stop: oneshot::Sender<()>,
    done: oneshot::Receiver<()>,
}

impl LogTail {
    /// Returns the handle for the following and the two ends the following itself holds: the
    /// stop signal to watch and the mark to send when it has finished.
    pub fn new() -> (Self, oneshot::Receiver<()>, oneshot::Sender<()>) {
        let (stop_tx, stop_rx) = oneshot::channel();
        let (done_tx, done_rx) = oneshot::channel();
        (
            Self {
                stop: stop_tx,
                done: done_rx,
            },
            stop_rx,
            done_tx,
        )
    }

    /// A following that follows nothing.
    pub fn idle() -> Self {
        Self::new().0
    }

    /// Stops the following and waits for its last announcement, but not for ever.
    pub async fn finish(self, timeout: Duration) {
        let Self { stop, done } = self;
        drop(stop);
        let _ = tokio::time::timeout(timeout, done).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_tell_the_following_to_stop_when_finished() {
        let (tail, stop, done) = LogTail::new();

        let finishing = tokio::spawn(tail.finish(Duration::from_secs(5)));
        assert!(stop.await.is_err(), "the stop signal is a closed channel");
        done.send(()).unwrap();

        assert!(finishing.await.is_ok());
    }

    #[tokio::test]
    async fn should_stop_waiting_when_the_following_does_not_finish_in_time() {
        let (tail, _stop, _done) = LogTail::new();

        let waited = tokio::time::timeout(
            Duration::from_secs(5),
            tail.finish(Duration::from_millis(20)),
        )
        .await;

        assert!(waited.is_ok());
    }
}
