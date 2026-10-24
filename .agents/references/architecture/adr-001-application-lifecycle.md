# ADR-001: Application Startup Lifecycle & IPC Handshake Invariants

## Context
Orbit combines a Vue 3 desktop frontend with a native Rust backend (`src-tauri`). When the desktop app launches, Tauri initializes the Rust backend and frontend window, communicating via bidirectional IPC (`dispatch_engine` commands and event emitters).

## Invariant 1: Synchronous Mounting Before `init()`
In [src/main.ts](file:///d:/Projects/orbit/src/main.ts):
```typescript
// 1. Mount Vue app synchronously FIRST
app.mount('#app')

// 2. Initialize native service host SECOND
init()

// 3. Hydrate persistent stores asynchronously in the background
void Promise.allSettled([
  initAppearance(),
  tableFilterStore.init(),
  notificationStore.init(),
])
```

### Why?
- `App.vue`'s `onMounted()` hook registers listeners for core IPC broadcast events:
  - `events.on(OrbitEvents.EngineConnected)`
  - `events.on(OrbitEvents.ClustersUpdated)`
  - `events.on(OrbitEvents.ActiveClusterChanged)`
- If `init()` runs before `app.mount('#app')`, or if `app.mount()` is delayed by top-level `await` promises, the backend engine connects and broadcasts initial cluster state *before* frontend listeners exist.
- **Symptom if violated**: Initial clusters are permanently lost, the UI hangs on loading, or a false "engine connection timeout" error dialog appears.

## Invariant 2: Asynchronous Persistent Store Hydration
- Never `await` persistent store hydration (e.g. `tableFilterStore.init()`, `initTheme()`) before `app.mount('#app')`.
- All storage hydration must run in the background using `void Promise.allSettled(...)`.
- Stores must provide sensible default reactive state immediately so initial views can render without blocking.

## Related Files
- [src/main.ts](file:///d:/Projects/orbit/src/main.ts)
- [src/App.vue](file:///d:/Projects/orbit/src/App.vue)
- [src/services/nativeService.ts](file:///d:/Projects/orbit/src/services/nativeService.ts)
- [core/engine/src/main.rs](file:///d:/Projects/orbit/core/engine/src/main.rs)
