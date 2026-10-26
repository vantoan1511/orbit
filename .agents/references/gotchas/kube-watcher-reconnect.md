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
