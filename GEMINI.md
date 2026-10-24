# GEMINI.md — Orbit Workspace Context

## 1. Project Overview

Orbit is a lightweight, high-performance native desktop Kubernetes dashboard.

### Technology Stack
- **Desktop Runtime**: Tauri v2
- **Backend Engine**: Rust (`src-tauri/`)
- **Frontend**: Vue 3 (Composition API + `<script setup>`), TypeScript, Vite
- **Styling & UI**: Tailwind CSS v4, PrimeVue v4 (Nora Noir Theme), PrimeIcons
- **State & Routing**: Pinia, Vue Router

The frontend communicates with the Rust backend exclusively through Tauri IPC (`dispatch_engine` commands and event emitters). Rust manages all privileged operations (Kubernetes APIs, OS filesystem, background processes), while Vue presents data and handles user interactions.

---

## 2. SDLC & Shift-Left Engineering (`my-harness`)

Orbit strictly adheres to the **Shift-Left Quality & Architecture Constitution** defined in `my-harness` (`rules/AGENTS.md`):
- **Branch Protection**: Never commit directly to `main` or `master`. Work on dedicated feature/fix branches.
- **Contract-First Design**: Define Rust IPC request/response data models and TypeScript interfaces before writing business logic.
- **Test-First (TDD)**: Write failing tests in `src/**/__tests__/*.test.ts` (Node native test runner) or `src-tauri` (`#[test]`) before writing implementation code.
- **Micro-Validation**: Run the pre-flight verification tool after every single file edit:
  ```bash
  python .agents/scripts/verify_shift_left.py --changed
  ```
- **Standard Workflows**:
  - `/specify`: Reconnaissance existing patterns and generate contract-first spec in `.agents/PRPs/plans/`.
  - `/prp-plan`: Structured architecture planning with atomic TDD task checklists.
  - `/implement`: Autonomous execution of the plan with continuous micro-validation loops.
  - `/code-review`: Shift-left audit of uncommitted diffs or GitHub PRs.
  - `/commit` & `/pr`: Conventional staging and PR generation.

---

## 3. Architecture & Boundary Invariants

### Backend (`src-tauri`)
- Owns all Kubernetes communication, kubeconfig parsing, background processes (e.g. port forwards), filesystem persistence (`~/.orbit/`), and business logic.
- **Rust Safety**: Zero `unwrap()` or `expect()` in production code. Handle errors explicitly via `Result<T, E>` and `?`. Return structured machine-readable error codes across the IPC boundary.

### Frontend (`src`)
- Purely representational: renders UI, manages local view state, and handles user interactions.
- **IPC & Boundary Discipline**:
  - **Zero direct imports of `@tauri-apps/api` or `@tauri-apps/plugin-*`** in Vue components, views, or composables.
  - All native, OS, windowing, and IPC operations must route through domain services (`@/services/*`, e.g. `nativeService`, `appSettingsService`).
- **Startup Lifecycle Sequence** ([ADR-001](file:///d:/Projects/orbit/.agents/references/architecture/adr-001-application-lifecycle.md)):
  - `app.mount('#app')` in `src/main.ts` must execute synchronously before `init()`.
  - Store hydration from native storage must occur asynchronously in the background (`void Promise.allSettled(...)`) after mounting. Never block `app.mount('#app')` with top-level `await`.
- **Storage Hierarchy** ([ADR-003](file:///d:/Projects/orbit/.agents/references/architecture/adr-003-storage-and-state.md)):
  - **Global Config**: Owned by Rust `OrbitConfig`, saved to `~/.orbit/config.json`.
  - **App State**: Persisted via `nativeService.storage` in `.storage/`. Never use browser `localStorage` or `sessionStorage` in production code. Always wrap disk storage calls with `withTimeout()`.
  - **Single Source of Truth**: Kubernetes live data lives exclusively in `useKubernetesStore`.

---

## 4. Specialized Workspace Rules & Skills

Orbit delegates detailed guidelines to specialized workspace rules and skills:

### Rules (`.agents/rules/`)
- [`orbit-design-system.md`](.agents/rules/orbit-design-system.md): Authoritative visual constitution. Noir monochrome palette, typography scale, information density, and K8s status colors.
- [`orbit-primevue.md`](.agents/rules/orbit-primevue.md): PrimeVue v4 Nora theme conventions, PassThrough rules, and single-source-of-truth component styling.
- [`orbit-frontend-engineering.md`](.agents/rules/orbit-frontend-engineering.md): Component reuse guidelines, PrimeVue component/v-tooltip enforcement, and settings partitioning.
- [`orbit-configuration-system.md`](.agents/rules/orbit-configuration-system.md): Centralized system-defined configuration model across Rust and Vue.

### Domain Skills (`.agents/skills/`)
- `orbit-resource-page`: Standard pattern for adding/editing Kubernetes resource pages (Deployments, Pods, Services, etc.).
- `orbit-primevue-customization`: Customizing PrimeVue components using Nora theme tokens.
- `orbit-ui-implementation`: Building UI screens matching Orbit's technical desktop IDE aesthetic.
- `orbit-ui-review`: Consistency review gate for visual fidelity and token compliance.
- `winget`: Windows Package Manager distribution and release automation.

---

## 5. Definition of Done

A task is complete only when:
- The requested behavior is implemented according to repository evidence discovered via `codegraph_explore`.
- Existing components and patterns were evaluated for reuse before introducing new code.
- PrimeVue v4 Nora controls and `v-tooltip` directives are used (no raw `<button>`, `<input>`, `<select>` or native `title="..."`).
- Design tokens from `src/assets/base.css` are used (no hardcoded hex colors).
- Shift-left scanner passes: `python .agents/scripts/verify_shift_left.py --changed`.
- Fast unit tests pass: `npm test` (and `cargo test` if backend modified).
- Type checking passes: `npm run type-check`.
- Linting passes: `npm run lint`.
- Final git diff is atomic and contains only changes within requested scope.
