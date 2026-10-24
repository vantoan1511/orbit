---
trigger: always_on
---

# Orbit Frontend Engineering

## Scope

These rules apply to Orbit's Vue 3 + Tauri v2 frontend.

## General Principles

Do not optimize for the shortest path if it creates:

- duplicated components
- duplicated styles
- inconsistent APIs
- page-specific hacks
- unnecessary abstractions
- new patterns that already exist elsewhere

Prefer consistency and maintainability over implementation speed.

## Reuse Before Create

Before creating a component:

1. Use `codegraph` (`codegraph_explore`) to survey existing components, composables, and patterns across the codebase.
2. Search for an existing PrimeVue configuration or wrapper.
3. Search for an analogous page.
4. Determine whether the existing implementation can be extended.

Only create a new component when reuse or extension would make the existing abstraction worse.

## Styling

Prefer, in order:

1. existing PrimeVue/Nora theme configuration
2. existing Orbit design tokens
3. shared component styles
4. existing utility classes
5. component-scoped styles for genuinely local layout
6. one-off global CSS only when explicitly justified

Do not hard-code repeated visual values when an existing token exists.

Do not copy a CSS block from another component and modify a few values. Extract or reuse the shared pattern instead.

## Page Components

Pages should primarily compose:

- existing layout components
- existing PrimeVue components
- shared Orbit components
- page-specific data and behavior

Avoid turning pages into styling laboratories.

## PrimeVue Components & Directives

- Use PrimeVue v4 components and directives for all controls and interactions.
- Always use PrimeVue's `v-tooltip` directive (e.g. `v-tooltip.top="'...'"` or `v-tooltip="'...'"`). Never use native HTML `title` attributes for tooltips or descriptive hover text.

## Native & IPC Boundaries

- **Never import `@tauri-apps/api` or legacy `@neutralinojs/lib` directly** inside components, views, or composables.
- Always access OS, filesystem, window, events, storage, and IPC functions via `@/services/nativeService` or specialized domain services (`@/services/*`).
- Components must remain purely representational and delegate system interactions to services or stores.

### Settings & Preferences Persistence Guardrail

- **Strict Prohibition on Ad-Hoc Storage Keys**:
  **NEVER** persist application settings, shortcuts, or preferences using isolated `storage.setData(...)` keys (e.g. `orbit_keyboard_shortcuts`, `orbit_appearance_preferences`, `orbit-settings.json`).
- **Single Source of Truth**:
  All application configurations and user customizations must reside in `~/.orbit/config.json`, defined centrally in the backend (`Configuration::system_definitions()`), and accessed via `settingsStore` / `appSettingsService`.
- **Specialized Settings Partitioning**:
  When introducing settings with dedicated UI tabs (e.g., `keyboardShortcuts` in `Preferences`, `themeMode` / `accentColor` / `uiStyle` in `Appearance`):
  1. Register the key in `CONFIG_KEYS` in `src/types/settings.ts`.
  2. Add the key to `SPECIALIZED_CONFIG_KEYS` in `src/types/settings.ts` so `GeneralSettingsForm.vue` automatically filters it out.
  3. Wire the dedicated tab component to `settingsStore` or its domain store/service.
- **Legacy Migration**:
  When migrating existing or deprecated local storage keys, automatically read the legacy key once, migrate the value via `appSettingsService.updateAppSettings(...)`, and remove the legacy key from storage.

## Application Lifecycle & Startup Sequence

- **Synchronous App Mounting Before `init()`**:
  `app.mount('#app')` in `src/main.ts` MUST execute synchronously before `init()` is called.
- **Why**: `App.vue`'s `onMounted()` hook registers core IPC event listeners (`events.on(OrbitEvents.EngineConnected)`, `events.on(OrbitEvents.ClustersUpdated)`, `events.on(OrbitEvents.ActiveClusterChanged)`). If `init()` runs before `app.mount('#app')` or if `app.mount('#app')` is delayed by asynchronous `await` calls, the backend engine connects and broadcasts initial state events before listeners exist. This causes initial clusters to be lost and triggers an engine connection timeout dialog.
- **Asynchronous Storage & Store Hydration**:
  Hydrate persistent stores and settings (e.g. `initTheme()`, `tableFilterStore.init()`, `notificationStore.init()`) asynchronously in the background (`void Promise.allSettled(...)`) AFTER `app.mount('#app')` and `init()`. Never block `app.mount('#app')` with top-level `await` promises.

## Verification

After implementation:

- inspect the changed component
- inspect analogous components
- search for duplicate selectors/classes introduced by the change
- verify responsive/desktop behavior
- run the relevant lint/typecheck/test/build commands when available

For UI changes, visual verification is required when the environment supports browser inspection.
