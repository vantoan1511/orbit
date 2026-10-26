# Gotcha: `kube::runtime::watcher` Reconnection Tight-Loop

## Problem

When using `kube::runtime::watcher(api, watcher::Config::default())` without `.default_backoff()` from `kube::runtime::WatchStreamExt`, any network disconnection, API server unreachability, or TLS error causes the stream to immediately yield error results in a tight unthrottled loop without delay.
This results in thousands of failed HTTP requests per second per watcher (20+ watchers yielding ~50,000 requests/sec), exhausting CPU, file descriptors, and flooding system logs.

## Solution

Always chain `.default_backoff()` onto the watcher stream:

```rust
use kube::runtime::{watcher, WatchStreamExt};

let mut stream = watcher(api, watcher::Config::default())
    .default_backoff()
    .boxed();
```

`default_backoff()` applies exponential backoff with jitter (starting at ~0.8s, up to ~30s), ensuring retries remain strictly bounded and preventing hot reconnect loops.

## Connection Health Semantics

Orbit monitors cluster watcher health through `WatcherHealthTracker` using partitioned status sets:

- **Healthy Set**: Watchers that have completed initial listing or processed an event (`Apply`, `Delete`, `InitDone`). Idle watchers remain in this set until an error occurs—Kubernetes resources may have zero mutations for hours, so lack of events must not be treated as a failure.
- **Failing Set (Unreachable)**: Watchers experiencing transport, TLS, connection timeout, or HTTP 5xx errors. The cluster transitions to `Degraded` only when the `failing` set is non-empty AND the `healthy` set is empty.
- **Rejected Set (RBAC / 401, 403, 404)**: When a cluster restricts permissions (e.g., read access denied to `Secrets` or `PersistentVolumes`) or an optional API is missing, the API server returns 401, 403, or 404. These are classified as `Rejected` and excluded from aggregate cluster degradation. A single missing permission must never trigger cluster suspension or cancel other healthy watchers.
- **Give-Up & Suspension**: Only when all active watchers are failing continuously with transport/unreachable errors for `>= WATCHER_GIVE_UP_AFTER` (5 minutes) does the tracker transition to `Suspended` and fire `cancel_tx`.
- **Reconnect via `restartWatchers`**: When recovering from disconnection, Orbit dispatches `restartWatchers` rather than `switchCluster`. This lightweight IPC command cleanly aborts old watcher loops and spawns fresh watchers/metrics pollers without destroying interactive pty terminals, active port-forwards, or streaming logs.

