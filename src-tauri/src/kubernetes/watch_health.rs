use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};
use tokio::sync::watch;

use crate::ipc::events::WatcherConnectionState;
use crate::kubernetes::watch_error::WatcherFailureClass;

pub const WATCHER_GIVE_UP_AFTER: Duration = Duration::from_secs(300);

struct WatcherHealthInner {
    healthy: HashSet<String>,
    failing: HashSet<String>,
    rejected: HashSet<String>,
    total_failures: usize,
    degraded_since: Option<Instant>,
    state: WatcherConnectionState,
}

impl WatcherHealthInner {
    fn recompute(&mut self, now: Instant) -> Option<WatcherConnectionState> {
        if self.state == WatcherConnectionState::Suspended {
            return None;
        }

        let next_state = if !self.failing.is_empty() && self.healthy.is_empty() {
            WatcherConnectionState::Degraded
        } else {
            WatcherConnectionState::Connected
        };

        match next_state {
            WatcherConnectionState::Degraded => {
                if self.state != WatcherConnectionState::Degraded {
                    self.degraded_since = Some(now);
                }
            }
            WatcherConnectionState::Connected | WatcherConnectionState::Suspended => {
                self.degraded_since = None;
            }
        }

        if next_state != self.state {
            self.state = next_state;
            Some(next_state)
        } else {
            None
        }
    }
}

pub struct WatcherHealthTracker {
    inner: Mutex<WatcherHealthInner>,
    give_up_after: Duration,
    /// Cancels the whole cluster watcher set, including the metrics poller.
    cancel_tx: watch::Sender<bool>,
}

impl WatcherHealthTracker {
    pub fn new(give_up_after: Duration, cancel_tx: watch::Sender<bool>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(WatcherHealthInner {
                healthy: HashSet::new(),
                failing: HashSet::new(),
                rejected: HashSet::new(),
                total_failures: 0,
                degraded_since: None,
                state: WatcherConnectionState::Connected,
            }),
            give_up_after,
            cancel_tx,
        })
    }

    pub fn record_failure(
        &self,
        watcher_id: &str,
        class: WatcherFailureClass,
        now: Instant,
    ) -> Option<WatcherConnectionState> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.total_failures += 1;
        if inner.state == WatcherConnectionState::Suspended {
            return None;
        }

        inner.healthy.remove(watcher_id);
        match class {
            WatcherFailureClass::Rejected => {
                inner.failing.remove(watcher_id);
                inner.rejected.insert(watcher_id.to_string());
            }
            WatcherFailureClass::Unreachable => {
                inner.rejected.remove(watcher_id);
                inner.failing.insert(watcher_id.to_string());
            }
        }

        inner.recompute(now)
    }

    pub fn record_success(&self, watcher_id: &str, now: Instant) -> Option<WatcherConnectionState> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if inner.state == WatcherConnectionState::Suspended {
            return None;
        }

        if inner.healthy.contains(watcher_id)
            && !inner.failing.contains(watcher_id)
            && !inner.rejected.contains(watcher_id)
        {
            return None;
        }

        inner.failing.remove(watcher_id);
        inner.rejected.remove(watcher_id);
        inner.healthy.insert(watcher_id.to_string());

        inner.recompute(now)
    }

    pub fn check_give_up(&self, now: Instant) -> Option<WatcherConnectionState> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        match inner.state {
            WatcherConnectionState::Degraded => {
                if let Some(degraded_since) = inner.degraded_since
                    && now.saturating_duration_since(degraded_since) >= self.give_up_after
                {
                    inner.state = WatcherConnectionState::Suspended;
                    if let Err(e) = self.cancel_tx.send(true) {
                        tracing::warn!(error = %e, "Failed to send cancel signal on watcher give-up");
                    }
                    return Some(WatcherConnectionState::Suspended);
                }
                None
            }
            WatcherConnectionState::Connected | WatcherConnectionState::Suspended => None,
        }
    }

    pub fn failing_count(&self) -> usize {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.failing.len()
    }

    pub fn rejected_count(&self) -> usize {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.rejected.len()
    }

    pub fn total_failure_records(&self) -> usize {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.total_failures
    }

    pub fn state(&self) -> WatcherConnectionState {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_failure_transitions_to_degraded_once() {
        let (tx, _rx) = watch::channel(false);
        let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, tx);
        let t0 = Instant::now();

        let res1 = tracker.record_failure("pods:Pod", WatcherFailureClass::Unreachable, t0);
        assert_eq!(res1, Some(WatcherConnectionState::Degraded));

        for i in 2..=5 {
            let res = tracker.record_failure(&format!("res{}:Kind", i), WatcherFailureClass::Unreachable, t0);
            assert_eq!(res, None);
        }

        assert_eq!(tracker.failing_count(), 5);
        assert_eq!(tracker.state(), WatcherConnectionState::Degraded);
    }

    #[test]
    fn rejected_failures_never_degrade_or_suspend() {
        let (tx, rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        let res = tracker.record_failure("secrets", WatcherFailureClass::Rejected, t0);
        assert_eq!(res, None);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);
        assert_eq!(tracker.rejected_count(), 1);
        assert_eq!(tracker.failing_count(), 0);

        let res_give_up = tracker.check_give_up(t0 + Duration::from_secs(100));
        assert_eq!(res_give_up, None);
        assert!(!*rx.borrow());
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);
    }

    #[test]
    fn one_unreachable_among_healthy_never_suspends() {
        let (tx, rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_success("pods", t0);
        let res = tracker.record_failure("jobs", WatcherFailureClass::Unreachable, t0);
        assert_eq!(res, None);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);

        let res_give_up = tracker.check_give_up(t0 + give_up);
        assert_eq!(res_give_up, None);
        assert!(!*rx.borrow());
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);
    }

    #[test]
    fn all_failing_unreachable_suspends_and_cancels() {
        let (tx, rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", WatcherFailureClass::Unreachable, t0);

        let res_before = tracker.check_give_up(t0 + Duration::from_secs(9));
        assert_eq!(res_before, None);
        assert!(!*rx.borrow());

        let res_at = tracker.check_give_up(t0 + Duration::from_secs(10));
        assert_eq!(res_at, Some(WatcherConnectionState::Suspended));
        assert!(*rx.borrow());
        assert_eq!(tracker.state(), WatcherConnectionState::Suspended);
    }

    #[test]
    fn healthy_watcher_failing_with_transport_error_degrades_when_none_healthy() {
        let (tx, _rx) = watch::channel(false);
        let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, tx);
        let t0 = Instant::now();

        tracker.record_success("pods", t0);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);

        let res = tracker.record_failure("pods", WatcherFailureClass::Unreachable, t0);
        assert_eq!(res, Some(WatcherConnectionState::Degraded));
        assert_eq!(tracker.state(), WatcherConnectionState::Degraded);
    }

    #[test]
    fn success_from_any_watcher_recovers_from_degraded() {
        let (tx, _rx) = watch::channel(false);
        let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", WatcherFailureClass::Unreachable, t0);
        tracker.record_failure("watcher-b", WatcherFailureClass::Unreachable, t0);
        assert_eq!(tracker.state(), WatcherConnectionState::Degraded);

        let res_a = tracker.record_success("watcher-a", t0);
        assert_eq!(res_a, Some(WatcherConnectionState::Connected));
        assert_eq!(tracker.failing_count(), 1);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);

        let res_b = tracker.record_success("watcher-b", t0);
        assert_eq!(res_b, None);
        assert_eq!(tracker.failing_count(), 0);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);
    }

    #[test]
    fn rejected_then_success_clears_rejected() {
        let (tx, _rx) = watch::channel(false);
        let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", WatcherFailureClass::Rejected, t0);
        assert_eq!(tracker.rejected_count(), 1);

        tracker.record_success("watcher-a", t0);
        assert_eq!(tracker.rejected_count(), 0);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);
    }

    #[test]
    fn suspended_state_is_sticky() {
        let (tx, _rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", WatcherFailureClass::Unreachable, t0);
        tracker.check_give_up(t0 + Duration::from_secs(10));
        assert_eq!(tracker.state(), WatcherConnectionState::Suspended);

        // late success does not resurrect
        let res_success = tracker.record_success("watcher-a", t0);
        assert_eq!(res_success, None);
        assert_eq!(tracker.state(), WatcherConnectionState::Suspended);

        // further check_give_up returns None
        let res_check = tracker.check_give_up(t0 + Duration::from_secs(20));
        assert_eq!(res_check, None);
    }

    #[test]
    fn recovery_resets_give_up_clock() {
        let (tx, rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", WatcherFailureClass::Unreachable, t0);
        tracker.record_success("watcher-a", t0);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);

        let t1 = t0 + Duration::from_secs(50);
        tracker.record_failure("watcher-a", WatcherFailureClass::Unreachable, t1);

        // At t1 + 8s, 58s from t0, but only 8s from t1 -> should not give up
        let res = tracker.check_give_up(t1 + Duration::from_secs(8));
        assert_eq!(res, None);
        assert!(!*rx.borrow());

        // At t1 + 10s -> gives up
        let res2 = tracker.check_give_up(t1 + Duration::from_secs(10));
        assert_eq!(res2, Some(WatcherConnectionState::Suspended));
        assert!(*rx.borrow());
    }

    #[test]
    fn cancel_send_failure_does_not_panic() {
        let (tx, rx) = watch::channel(false);
        drop(rx);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", WatcherFailureClass::Unreachable, t0);
        let res = tracker.check_give_up(t0 + give_up);
        assert_eq!(res, Some(WatcherConnectionState::Suspended));
        assert_eq!(tracker.state(), WatcherConnectionState::Suspended);
    }
}
