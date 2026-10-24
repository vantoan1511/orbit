# Gotchas: Tauri v2 Runtime & IPC Invariants

---

## 1. Direct Imports of `@tauri-apps/api`
- **NEVER** import `@tauri-apps/api` or invoke Tauri commands directly inside Vue components, views, or composables (`src/components/`, `src/views/`, `src/composables/`).
- **Why**: Direct imports break component testability, violate boundary separation, and cause test suites to fail in Node.js where Tauri's IPC bridge is not present.
- **Rule**: Always import via `@/services/nativeService` or dedicated domain services in `src/services/*`.

---

## 2. Asynchronous Native Storage Timeouts
- Calling native file or storage APIs makes an IPC request to Tauri plugins/OS filesystem.
- On Windows or slow disk environments, if storage hangs or is corrupt, UI initialization can block indefinitely.
- **Rule**: Always use the `withTimeout` wrapper ([src/utils/async.ts](file:///d:/Projects/orbit/src/utils/async.ts)) with `DEFAULT_STORAGE_TIMEOUT_MS` (1500ms) to fall back safely to defaults.

---

## 3. Background Process Management & Port Forwards
- Port forwards spawn background `kubectl` processes managed by the Rust backend engine (`src-tauri`).
- On exit or stop, processes must be killed cleanly.
- Windows process termination requires handling process trees (`taskkill /F /T /PID <pid>`) so orphaned child connections don't hold ports open.
