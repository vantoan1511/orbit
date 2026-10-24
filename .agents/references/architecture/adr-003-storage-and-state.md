# ADR-003: Storage Hierarchy & State Boundaries

## Context
As a native desktop application built with Tauri v2, Rust, and Vue 3, Orbit operates under different rules than standard browser single-page applications.

## Storage Tiers & Responsibilities

| Tier | Technology | Persistence Location | Purpose | Examples |
|---|---|---|---|---|
| **Tier 1: Global App Config** | Rust `OrbitConfig` | `~/.orbit/config.json` | System settings, user preferences, telemetry, security settings | `maxLogFiles`, custom cluster aliases |
| **Tier 2: Native App Storage** | Native file storage via `nativeService` | `.storage/` directory | User state across sessions, table column widths, table filter presets, notifications | `orbit_table_filters`, `orbit-theme-preference`, sidebar width |
| **Tier 3: Reactive Memory** | Pinia Stores (`src/stores/*`) | In-memory RAM | Live Kubernetes resources, active connection state, search query, active drawer | Pods, Deployments, search modal state |
| **Tier 4: Component Local** | Vue `ref()` / `reactive()` | In-memory RAM | Ephemeral form edits, local tab selection, dropdown visibility | Form inputs before save |

## Non-Negotiable Rules

1. **NEVER use browser `localStorage` or `sessionStorage` in production code**:
   - Webview `localStorage` is subject to domain isolation, sandbox clearing, and web engine cache resets.
   - Always persist desktop user preferences using `nativeService.storage` or Rust `OrbitConfig`.
   - Legacy migration: If reading a key, check `nativeStorage` first; if absent, read `localStorage` as a one-time migration and immediately save to `nativeStorage`.
2. **Always Wrap Native Storage with Timeouts**:
   - Native storage disk operations are asynchronous IPC/filesystem calls.
   - Always use `withTimeout(storage.getData(key), DEFAULT_STORAGE_TIMEOUT_MS)` to ensure an unresponsive disk operation falls back safely to default values within 1.5 seconds.
3. **Never duplicate backend data across multiple Pinia stores**:
   - Single source of truth for Kubernetes data lives in `useKubernetesStore`. Other stores reference or compute derived views over it.
