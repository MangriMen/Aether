use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// How far back the speed of a transfer looks: long enough to smooth out uneven chunks, short
/// enough to follow a change of pace within a couple of seconds.
pub(crate) const SPEED_WINDOW: Duration = Duration::from_secs(2);

/// Reports closer together than this share one sample, so a fast transfer of many small chunks
/// keeps a few dozen samples rather than thousands.
const SAMPLE_RESOLUTION: Duration = Duration::from_millis(50);

/// The recent history of a bar's transferred bytes, for its speed.
#[derive(Debug, Clone, Default)]
pub(crate) struct TransferWindow {
    /// `(when, bytes done by then)`, oldest first. Once the history is long enough, the oldest
    /// sample is at or before the start of the window, so the speed covers all of it.
    samples: VecDeque<(Instant, u64)>,
}

impl TransferWindow {
    /// Note that `bytes_done` bytes have been transferred by `now`.
    pub(crate) fn record(&mut self, now: Instant, bytes_done: u64) {
        let len = self.samples.len();
        if len >= 2 && now.saturating_duration_since(self.samples[len - 2].0) < SAMPLE_RESOLUTION {
            self.samples[len - 1] = (now, bytes_done);
        } else {
            self.samples.push_back((now, bytes_done));
        }
        self.prune(now);
    }

    /// Bytes per second over the last [`SPEED_WINDOW`] up to `now`: it falls to zero within the
    /// window once the bytes stop. `None` before there are two moments to measure between.
    pub(crate) fn speed(&mut self, now: Instant) -> Option<f64> {
        self.prune(now);
        let &(first_at, first_bytes) = self.samples.front()?;
        let &(_, last_bytes) = self.samples.back()?;

        let elapsed = now.saturating_duration_since(first_at).as_secs_f64();
        if elapsed <= 0.0 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)] // a speed to show
        let bytes = last_bytes.saturating_sub(first_bytes) as f64;
        Some(bytes / elapsed)
    }

    /// Drop the samples older than the window, keeping the newest of them as its start.
    fn prune(&mut self, now: Instant) {
        let Some(start) = now.checked_sub(SPEED_WINDOW) else {
            return;
        };
        while self.samples.len() >= 2 && self.samples[1].0 <= start {
            self.samples.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn has_no_speed_without_two_moments_to_measure_between() {
        let start = Instant::now();
        let mut window = TransferWindow::default();
        assert_eq!(window.speed(start), None);

        window.record(start, 1000);

        assert_eq!(window.speed(start), None);
    }

    #[test]
    fn measures_bytes_per_second_over_the_last_two_seconds() {
        let start = Instant::now();
        let mut window = TransferWindow::default();

        window.record(start, 1000);
        window.record(start + ms(500), 2000);
        assert_eq!(window.speed(start + ms(500)), Some(2000.0));

        window.record(start + ms(1000), 3000);
        assert_eq!(window.speed(start + ms(1000)), Some(2000.0));

        // The window now starts at the 500 ms sample: the first one no longer counts.
        window.record(start + ms(2500), 7000);
        assert_eq!(window.speed(start + ms(2500)), Some(2500.0));
    }

    #[test]
    fn falls_to_zero_once_the_bytes_stop() {
        let start = Instant::now();
        let mut window = TransferWindow::default();
        window.record(start, 0);
        window.record(start + ms(1000), 1000);

        assert_eq!(window.speed(start + ms(2000)), Some(500.0));
        assert_eq!(window.speed(start + ms(3000)), Some(0.0));
    }

    #[test]
    fn keeps_one_sample_for_reports_close_together() {
        let start = Instant::now();
        let mut window = TransferWindow::default();

        window.record(start, 100);
        for step in 1..=100 {
            window.record(start + ms(step), 100 + step * 10);
        }

        // About one sample per 50 ms, each a real moment: the speed stays exact.
        assert!(window.samples.len() <= 4, "{:?}", window.samples);
        let speed = window.speed(start + ms(100)).unwrap();
        assert!((speed - 10_000.0).abs() < 1e-6, "{speed}");
    }
}
