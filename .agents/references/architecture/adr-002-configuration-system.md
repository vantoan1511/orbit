# ADR-002: Backend-Controlled Unified Configuration System

## Context
Orbit previously had fragmented settings stored in multiple places (browser localStorage, hardcoded constants). This ADR establishes a single source of truth for all application configurations.

## Architecture Decisions

1. **System-Controlled Definitions in Rust (`core/engine/src/config.rs`)**:
   - Every application setting is formally declared in `Configuration::system_definitions()`.
   - Metadata includes: `key`, `name`, `description`, `datatype` (`string|number|boolean`), `defaultValue`, `cardinality` (`0..1|1..1|0..*|1..*`), `enable`, and `isConfidential`.
   - The frontend never hardcodes available settings or schemas.

2. **Lean Disk Persistence (`~/.orbit/config.json`)**:
   - The user's configuration file on disk strictly stores custom key-value overrides:
     ```json
     {
       "maxLogFiles": 10,
       "theme": "dark"
     }
     ```
   - Defaults are never written to disk unless explicitly changed by the user.

3. **Dynamic Frontend Rendering**:
   - [src/components/settings/GeneralSettingsForm.vue](file:///d:/Projects/orbit/src/components/settings/GeneralSettingsForm.vue) dynamically renders PrimeVue controls based on configuration metadata:
     - `datatype === 'boolean'` → `ToggleSwitch`
     - `datatype === 'number'` → `InputNumber` or preset `Select`
     - `datatype === 'string'` → `InputText`
     - `isConfidential === true` → Base64 encoded in transit, masked on UI.

4. **IPC Contract**:
   - Dispatched from frontend: `getAppSettings`, `updateAppSettings`.
   - Emitted by backend: `appSettingsUpdated { settings: Vec<Configuration> }`.

## Adding a New Setting (Quick Guide)
1. Add definition to `Configuration::system_definitions()` in [core/engine/src/config.rs](file:///d:/Projects/orbit/core/engine/src/config.rs).
2. Read in backend via `OrbitConfig::load().values.get("yourKey")`.
3. Read in frontend via `settingsStore.getConfigValue('yourKey', defaultValue)`.
4. The UI in GeneralSettingsForm will render it automatically without touching Vue code.
