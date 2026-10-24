# Orbit Configuration System

## Overview

Orbit uses a centralized, unified, and system-defined configuration model managed by the Rust backend (`src-tauri`) and delivered to the Vue 3 frontend via Tauri v2 IPC.

### Core Principles

1. **System-Controlled Definitions**: Application configurations are defined, validated, and controlled by the system (delivered in the backend package).
2. **Unified Data Contract**: Backend and frontend communicate exclusively via the unified `Configuration` model.
3. **User Customization Scope**: Users can only modify the `value` of enabled configurations.
4. **Lean Local Storage**: The user's local configuration file (`~/.orbit/config.json`) strictly persists customized key-value pairs (`{"key": value}`).
5. **Dynamic UI Rendering**: The frontend settings views dynamically render appropriate controls (`ToggleSwitch`, `InputNumber`, `Select`, `InputText`) based on configuration metadata (`datatype`, `coordinary`, `enable`, `isConfidential`).

---

## Storage & File Hierarchy

All Orbit application runtime files and user preferences reside in the user's home directory (`~/.orbit`):

- Configuration file: `~/.orbit/config.json` (key-value overrides only)
- Logs directory: `~/.orbit/logs/`
  - Active log: `~/.orbit/logs/orbit.log`
  - Daily archives: `~/.orbit/logs/orbit.log.YYYY-MM-DD`

---

## Data Model Specification

```typescript
export type ConfigurationDataType = 'string' | 'number' | 'boolean'
export type ConfigurationCardinality = '0..1' | '0..*' | '1..*' | '1..1' | string

export interface Configuration<T = unknown> {
  key: string                  // Unique identification key (e.g., 'maxLogFiles')
  name: string                 // Display title shown on UI
  description: string          // Detailed description of the setting
  datatype: ConfigurationDataType // 'string' | 'number' | 'boolean'
  defaultValue?: T             // Default fallback value
  value?: T                    // Effective or user-customized value
  isConfidential: boolean      // When true, encoded in transit (Base64)
  cardinality: ConfigurationCardinality // Multiplicity ('0..1', '0..*', '1..*', '1..1')
  enable: boolean              // Applicable/active flag (false = disabled / Coming soon)
  createdAt?: string           // ISO timestamp of definition creation
  lastUpdatedAt?: string       // ISO timestamp of last update
}
```

### Cardinality / Multiplicity (`cardinality`)

| Cardinality | Meaning | UI Representation |
|---|---|---|
| `0..1` | Single optional value | Single input field (can be left blank) |
| `1..1` | Single mandatory value | Single required input control (`InputNumber`, `ToggleSwitch`, `InputText`) |
| `0..*` | Multiple optional values | Comma-separated list or item list |
| `1..*` | Multiple values (at least 1 required) | Multi-item list with validation |

---

## Architectural Layers

```
┌────────────────────────────────────────────────────────┐
│ Vue Frontend                                           │
│  - Settings UI: src/components/settings/*.vue          │
│    (Dynamic v-for rendering with PrimeVue controls)    │
│  - Reactive Store: src/stores/settingsStore.ts         │
│  - Typed IPC Service: src/services/appSettingsService.ts│
└──────────────────────────┬─────────────────────────────┘
                           │ Tauri v2 IPC (`dispatch_engine` / Events)
                           ▼
┌────────────────────────────────────────────────────────┐
│ Rust Engine (src-tauri)                                │
│  - IPC Dispatcher: src-tauri/src/ipc/handlers/mod.rs   │
│  - Settings Handler: src-tauri/src/ipc/handlers/settings.rs │
│  - Events Definition: src-tauri/src/ipc/events.rs      │
│  - Config Schema & Storage: src-tauri/src/config.rs    │
│  - File System Storage: ~/.orbit/config.json           │
└────────────────────────────────────────────────────────┘
```

---

## Backend (Rust)

### 1. Unified Configuration Struct (`src-tauri/src/config.rs`)

- `Configuration`: Represents individual setting metadata, default values, and effective values.
- `Configuration::system_definitions()`: Canonical source of truth defining all system configurations.
- `OrbitConfig`: Transparent map (`HashMap<String, Value>`) mapped to `~/.orbit/config.json`.
  - `.load()`: Loads key-value pairs from `config.json` with legacy alias normalization.
  - `.save()`: Serializes key-value pairs to `config.json`.
  - `.get_configurations()`: Combines system definitions with user values, automatically Base64-encoding confidential values.
  - `.update_from_configurations(&[Configuration])`: Updates user values, decoding Base64 if confidential.

### 2. IPC Handlers (`src-tauri/src/ipc/handlers/settings.rs`)

- `getAppSettings`: Reads `OrbitConfig::load().get_configurations()` and sends `OrbitEvent::AppSettingsUpdated`.
- `updateAppSettings`: Accepts `Vec<Configuration>` or `HashMap<String, Value>`, updates `OrbitConfig`, saves to disk, and broadcasts `OrbitEvent::AppSettingsUpdated`.

### 3. Events (`src-tauri/src/ipc/events.rs`)

- `AppSettingsUpdated { settings: Vec<Configuration> }`: Emitted whenever configuration is fetched or updated.

---

## Frontend (Vue 3 / TypeScript)

### 1. Type Definitions (`src/types/settings.ts`)

- `Configuration<T>`: Unified configuration interface matching Rust backend.
- `ConfigurationMap`: Key-value record (`Record<string, unknown>`).

### 2. Frontend Service (`src/services/appSettingsService.ts`)

- `getAppSettings()`: Dispatches `'getAppSettings'`.
- `updateAppSettings(settings: Configuration[] | ConfigurationMap)`: Dispatches `'updateAppSettings'`.

### 3. Pinia Store (`src/stores/settingsStore.ts`)

- Reactive state: `settings` (`ref<Configuration[]>`), `isLoading`, `isSaving`.
- Helper getters:
  - `getConfig(key: string): Configuration | undefined`
  - `getConfigValue<T>(key: string, fallback?: T): T`
- Action helpers:
  - `updateConfigValue(key: string, value: unknown)`
  - `updateSettings(newSettings: Configuration[] | ConfigurationMap)`

### 4. Settings UI (`src/components/settings/`)

- `GeneralSettingsForm.vue`: Dynamically iterates over `settingsStore.settings`.
- Adheres to Orbit UI guidelines (PrimeVue v4, Nora theme, two-column borderless layout, `v-tooltip` for hints).
- Automatically renders:
  - `ToggleSwitch` for `datatype === 'boolean'`
  - `InputNumber` or preset `Select` for `datatype === 'number'`
  - `InputText` with multi-value formatting for `cardinality === '0..*' || cardinality === '1..*'`
  - Password-masked or confidential-badged inputs for `isConfidential === true`
  - Disabled styling (`opacity-60`, `Tag` with "Coming soon") when `enable === false`

---

## Adding New Settings

To add a new application configuration:

1. **Define in Backend**: Add the configuration item to `Configuration::system_definitions()` in `src-tauri/src/config.rs`:
   ```rust
   Configuration {
       key: "myNewSetting".to_string(),
       name: "My New Setting".to_string(),
       description: "Description of what this setting controls.".to_string(),
       datatype: "boolean".to_string(),
       default_value: Some(serde_json::json!(true)),
       value: None,
       is_confidential: false,
       cardinality: "1..1".to_string(),
       enable: true,
       created_at: Some("2026-09-01T00:00:00Z".to_string()),
       last_updated_at: None,
   }
   ```
2. **Register in Frontend Contracts (`src/types/settings.ts`)**:
   - Add the key to `CONFIG_KEYS`.
   - **If the setting has a dedicated tab** (e.g., `Preferences`, `Appearance`):
     Add the key to `SPECIALIZED_CONFIG_KEYS` so `GeneralSettingsForm.vue` automatically filters it out and does NOT render it as a generic form field.
   - **If the setting belongs in the General tab**:
     No form component edits needed — `GeneralSettingsForm.vue` dynamically renders the appropriate PrimeVue control (`ToggleSwitch`, `InputNumber`, `Select`, `InputText`).
3. **Access in Backend**: Use `OrbitConfig::load().values.get("myNewSetting")` or create a helper accessor on `OrbitConfig`.
4. **Access in Frontend**: Use `settingsStore.getConfigValue('myNewSetting', defaultValue)` or bind reactively via `settingsStore`.
5. **Update in Frontend**: Use `settingsStore.updateConfigValue('myNewSetting', newValue)` or `appSettingsService.updateAppSettings(...)`. **NEVER** use `storage.setData('orbit_myNewSetting', ...)`!
6. **Automatic Migration**: If migrating from legacy storage keys, detect legacy keys on load, dispatch `updateAppSettings`, and clean up `storage`.
7. **Testing**: Add or update unit tests in `src-tauri/src/config.rs` and frontend test suites (`src/types/__tests__/settings.test.ts`).
