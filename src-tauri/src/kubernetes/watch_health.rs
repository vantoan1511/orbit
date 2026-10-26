use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};
use tokio::sync::watch;

use crate::ipc::events::WatcherConnectionState;

pub const WATCHER_GIVE_UP_AFTER: Duration = Duration::from_secs(300);

struct WatcherHealthInner {
    failing: HashSet<String>,
    total_failures: usize,
    degraded_since: Option<Instant>,
    state: WatcherConnectionState,
}

pub struct WatcherHealthTracker {
    inner: Mutex<WatcherHealthInner>,
    give_up_after: Duration,
    suspend_tx: watch::Sender<bool>,
}

impl WatcherHealthTracker {
    pub fn new(give_up_after: Duration, suspend_tx: watch::Sender<bool>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(WatcherHealthInner {
                failing: HashSet::new(),
                total_failures: 0,
                degraded_since: None,
                state: WatcherConnectionState::Connected,
            }),
            give_up_after,
            suspend_tx,
        })
    }

    pub fn record_failure(&self, watcher_id: &str, now: Instant) -> Option<WatcherConnectionState> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.total_failures += 1;
        if inner.state == WatcherConnectionState::Suspended {
            return None;
        }

        inner.failing.insert(watcher_id.to_string());
        match inner.state {
            WatcherConnectionState::Connected => {
                inner.state = WatcherConnectionState::Degraded;
                inner.degraded_since = Some(now);
                Some(WatcherConnectionState::Degraded)
            }
            WatcherConnectionState::Degraded | WatcherConnectionState::Suspended => None,
        }
    }

    pub fn record_success(&self, watcher_id: &str) -> Option<WatcherConnectionState> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if inner.state == WatcherConnectionState::Suspended {
            return None;
        }

        inner.failing.remove(watcher_id);
        match inner.state {
            WatcherConnectionState::Degraded if inner.failing.is_empty() => {
                inner.state = WatcherConnectionState::Connected;
                inner.degraded_since = None;
                Some(WatcherConnectionState::Connected)
            }
            WatcherConnectionState::Degraded
            | WatcherConnectionState::Connected
            | WatcherConnectionState::Suspended => None,
        }
    }

    pub fn check_give_up(&self, now: Instant) -> Option<WatcherConnectionState> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        match inner.state {
            WatcherConnectionState::Degraded => {
                if let Some(degraded_since) = inner.degraded_since
                    && now.saturating_duration_since(degraded_since) >= self.give_up_after
                {
                    inner.state = WatcherConnectionState::Suspended;
                    let _ = self.suspend_tx.send(true);
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

        let res1 = tracker.record_failure("pods:Pod", t0);
        assert_eq!(res1, Some(WatcherConnectionState::Degraded));

        for i in 2..=5 {
            let res = tracker.record_failure(&format!("res{}:Kind", i), t0);
            assert_eq!(res, None);
        }

        assert_eq!(tracker.failing_count(), 5);
        assert_eq!(tracker.state(), WatcherConnectionState::Degraded);
    }

    #[test]
    fn recovery_of_all_watchers_returns_connected() {
        let (tx, _rx) = watch::channel(false);
        let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", t0);
        tracker.record_failure("watcher-b", t0);

        let res_a = tracker.record_success("watcher-a");
        assert_eq!(res_a, None);
        assert_eq!(tracker.failing_count(), 1);

        let res_b = tracker.record_success("watcher-b");
        assert_eq!(res_b, Some(WatcherConnectionState::Connected));
        assert_eq!(tracker.failing_count(), 0);
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);
    }

    #[test]
    fn give_up_after_window_suspends_and_cancels() {
        let (tx, rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", t0);

        let res_before = tracker.check_give_up(t0 + Duration::from_secs(9));
        assert_eq!(res_before, None);
        assert_eq!(*rx.borrow(), false);

        let res_at = tracker.check_give_up(t0 + Duration::from_secs(10));
        assert_eq!(res_at, Some(WatcherConnectionState::Suspended));
        assert_eq!(*rx.borrow(), true);
        assert_eq!(tracker.state(), WatcherConnectionState::Suspended);
    }

    #[test]
    fn suspended_state_is_sticky() {
        let (tx, _rx) = watch::channel(false);
        let give_up = Duration::from_secs(10);
        let tracker = WatcherHealthTracker::new(give_up, tx);
        let t0 = Instant::now();

        tracker.record_failure("watcher-a", t0);
        tracker.check_give_up(t0 + Duration::from_secs(10));
        assert_eq!(tracker.state(), WatcherConnectionState::Suspended);

        // late success does not resurrect
        let res_success = tracker.record_success("watcher-a");
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

        tracker.record_failure("watcher-a", t0);
        tracker.record_success("watcher-a");
        assert_eq!(tracker.state(), WatcherConnectionState::Connected);

        let t1 = t0 + Duration::from_secs(50);
        tracker.record_failure("watcher-a", t1);

        // At t1 + 8s, 58s from t0, but only 8s from t1 -> should not give up
        let res = tracker.check_give_up(t1 + Duration::from_secs(8));
        assert_eq!(res, None);
        assert_eq!(*rx.borrow(), false);

        // At t1 + 10s -> gives up
        let res2 = tracker.check_give_up(t1 + Duration::from_secs(10));
        assert_eq!(res2, Some(WatcherConnectionState::Suspended));
        assert_eq!(*rx.borrow(), true);
    }
}
