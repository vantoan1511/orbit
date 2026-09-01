# GEMINI.md

# Project

Orbit is a lightweight, native desktop Kubernetes dashboard.

Technology stack:

- Rust (backend)
- Neutralinojs (desktop runtime)
- Vue 3
- Tailwindcss v4
- PrimeVue v4
- TypeScript
- Vite

The frontend communicates with the Rust backend exclusively through Neutralinojs IPC. Rust is responsible for all privileged operations, while Vue is responsible for presenting data and handling user interactions.

---

# Agent Workflow

Agents must follow this workflow for every task.

## Phase 1 — Understand

Before modifying code:

1. Use `codegraph` (`codegraph_explore`) as the primary tool to explore architecture, survey components, and locate relevant symbols/flows.
2. Inspect the relevant existing implementation and neighboring components for established patterns.
3. Search for reusable components, composables, services, utilities, types, and IPC commands.
4. Inspect relevant design-system files for UI work.
5. Identify the smallest set of files that should change.

Do not begin implementation before completing this phase.

## Phase 2 — Plan

For non-trivial tasks:

1. Following workflow /plan to create or update the corresponding plan under `.agents/PRPs/plans/`.
2. Record:

   - problem

   - current implementation

   - proposed solution

   - files to modify

   - files to create

   - reusable existing code

   - risks

   - verification steps

3. Ensure the plan is based on repository evidence discovered during Phase 1.

Do not invent files, APIs, components, or architecture.

## Phase 3 — Implement

Implement only the requested scope.

- Start from existing patterns, but treat them as evidence rather than absolute rules.
- Reuse existing code when it represents the same underlying behavior.
- Generalize an existing implementation when the new requirement is the same concept with meaningful variation.
- Introduce a new abstraction or design pattern when the problem has a real responsibility boundary, multiple meaningful variants, independent changeability, or a concrete extensibility/testability need.
- Keep changes focused and minimal, but do not confuse minimal code with minimal complexity. Prefer the simplest correct design, not necessarily the fewest lines.
- Do not refactor unrelated code.
- Do not add speculative features or abstractions for hypothetical future requirements.
- Do not change architecture unless required or clearly justified by the task.
- Do not create local variants of shared components without a clear semantic reason.
- When intentionally deviating from an existing pattern, document the reason in the plan before implementation.

## Phase 4 — Verify

After implementation:

1. Run relevant tests.
2. Run type checking.
3. Run linting when configured.
4. Build the affected application/package when practical.
5. Inspect `git diff` and `git status`.
6. Verify that no unrelated files were modified.

If a verification step cannot be performed, explicitly report why.

Never claim verification that was not actually performed.

## Phase 5 — Report

The final response must contain:

- What changed
- Files changed
- Verification performed
- Remaining issues or limitations

Never claim that work was completed if required verification has not been performed.

---

# Repository Evidence

The repository is the source of truth.

Before referring to an existing component, composable, service, utility, IPC command, Rust module, type, route, store, or design token, locate and verify it in the repository using `codegraph` (`codegraph_explore`) or targeted searches.

Never assume an implementation exists because it would be conventional.

If an API or abstraction cannot be found:

1. Explore using `codegraph_explore` or search again using related terminology.
2. Confirm that it does not exist.
3. Propose the smallest required addition rather than assuming it exists.

Use existing implementation patterns as evidence for new code whenever possible.

---

# Architecture Judgment

## Do Not Confuse Consistency With Correctness

Consistency is a design constraint, not an absolute rule.

An existing implementation should be reused when it represents the same underlying problem. When the problem is materially different, do not force the new implementation into an existing pattern merely to make the code look consistent.

Prefer:

> **Consistent principles over identical implementations.**

Two components may legitimately use different internal designs when their responsibilities, change patterns, or complexity are different.

Agents are expected to exercise engineering judgment rather than mechanically mirror nearby code.

## Complexity Levels

Classify the problem before selecting the design:

| Level | Situation | Default behavior |
|---|---|---|
| **L1 — Simple** | One straightforward behavior with no meaningful variation | Follow the existing pattern |
| **L2 — Variable** | Same concept with multiple meaningful variations | Generalize, parameterize, or use a suitable strategy/policy |
| **L3 — Architectural** | Multiple strategies, responsibilities, boundaries, or independently evolving implementations | Introduce an appropriate abstraction or design pattern |

This classification is a guide, not a rigid rule. Use engineering judgment.

## Design Pattern Rule

Design patterns are tools, not goals.

An agent may introduce a recognized design pattern when it provides a concrete benefit such as:

- isolating changing behavior;
- supporting multiple strategies or implementations;
- reducing coupling;
- enforcing a meaningful boundary;
- improving testability;
- making extension safer.

Do not introduce a design pattern solely because it is theoretically applicable.

Likewise, do not avoid a design pattern solely because the repository does not currently use one.

## Architectural Deviation

When intentionally deviating from an existing repository pattern, the plan must briefly state:

- **Existing pattern** — what the repository currently does.
- **Problem** — why that pattern is insufficient for this requirement.
- **Alternative** — what design is proposed instead.
- **Trade-off** — what additional complexity is introduced and why it is acceptable.
- **Why now** — why the abstraction or pattern is justified at this point.

The implementation should make the deviation deliberate and easy for a reviewer to understand.

---

# Scope Discipline

Implement the requested change, not an imagined future version of it.

Do not:

- add speculative abstractions
- redesign unrelated UI
- rename unrelated variables
- reorganize directories without need
- upgrade dependencies unless requested or strictly required
- introduce additional features
- "clean up" unrelated code
- refactor neighboring code merely because it could be improved

If an improvement is discovered outside the requested scope, mention it in the final report instead of implementing it.

---

# Reuse and Abstraction Strategy

Existing code is evidence, not authority.

Before choosing an implementation, determine whether the new requirement is genuinely the same problem, the same concept with variation, or a materially different problem.

## 1. Reuse

Ask:

> Is this genuinely the same behavior?

If yes, reuse the existing implementation.

Prefer existing:

- feature implementations
- shared components
- composables
- services
- utilities
- PrimeVue components

## 2. Generalize

Ask:

> Is this the same fundamental behavior with meaningful variation?

If yes, consider generalizing the existing implementation rather than duplicating it.

Appropriate mechanisms may include:

- configuration
- parameterization
- strategy/policy objects
- generic components
- typed interfaces
- handler registries

## 3. Isolate

Ask:

> Does this introduce a new responsibility, boundary, or independently changing concern?

If yes, isolate that responsibility behind an appropriate abstraction.

## 4. Keep Local

If the behavior is unique, simple, and unlikely to benefit from meaningful variation, keep it feature-local.

## Abstraction Decision Criteria

A new abstraction is justified when one or more of the following provide a concrete benefit:

- multiple real implementations already exist;
- multiple implementations are explicitly required;
- behavior changes independently from its consumers;
- the abstraction protects an architectural boundary;
- the abstraction substantially improves testability or isolation;
- the current design would otherwise create substantial or growing duplication;
- extension is an explicit or strongly evidenced requirement;
- the existing pattern creates excessive coupling or responsibility.

Weak reasons for abstraction include:

- "we might need it someday";
- "this is more SOLID";
- "this is a common design pattern";
- "the code looks cleaner this way";
- "we could reuse this later".

Do not avoid an abstraction merely because the repository does not currently use one.

Do not introduce an abstraction merely because the code could theoretically be reused.

The goal is appropriate complexity for the problem.

---

# Git Workflow

For feature work and bug fixes:

1. Do not work directly on `main` or `master`.
2. Before modifying files, verify the current branch.
3. Create or checkout a dedicated branch before making changes.
4. Keep the branch scoped to the task.
5. Do not commit unrelated changes.
6. Do not rewrite existing commits unless explicitly requested.
7. Before finishing, inspect `git diff` and `git status`.

---

# Coding rules

Before generating code:

1. Inspect the existing architecture.
2. Follow established project conventions.
3. Reuse existing modules whenever possible.
4. Keep changes focused and minimal.
5. Avoid unrelated refactoring.
6. Never perform self-initiated or unrequested refactoring; touch existing code only when explicitly asked or strictly required for the requested change.
7. Explain trade-offs when multiple implementations are reasonable.
8. Always check-out to another branch for features implementing or bugs fixing, never work on main/master.
9. Keep code small and well-organized by business logic boundaries.
10. Design for **valuable** extensibility from the start: when the requirements or domain clearly indicate meaningful variation (such as rule evaluations, parsers, providers, or handler registries), choose an extensible design upfront. Do not add extension points solely for hypothetical future requirements.

Never invent APIs that do not exist.

If a required API is missing, propose adding it rather than assuming it already exists.

---

# Mission

Orbit should be:

- Lightweight
- Fast
- Native-feeling
- Reliable
- Secure
- Cross-platform
- Easy to maintain
- Production ready

Every implementation should favor simplicity, predictability, and long-term maintainability while allowing justified abstractions and extension points when the problem requires them.

---

# Architecture

Orbit is divided into two layers.

## Backend (Rust)

The Rust backend owns:

- Kubernetes API communication
- File system access
- Network access
- Authentication
- kubeconfig management
- Caching
- Background tasks
- Business logic
- Performance-critical operations
- System integration

The backend should not contain UI concerns.

---

## Frontend (Vue)

The frontend owns:

- Rendering
- User interactions
- View state
- Routing
- Local UI state
- Animations

The frontend should avoid implementing business logic.

Whenever logic requires Kubernetes knowledge or system access, it belongs in Rust.

---

# IPC

The IPC boundary is the contract between frontend and backend.

Always:

- keep commands small
- use strongly typed request/response models
- return structured errors
- avoid sending unnecessary data
- keep payloads versionable

Do not expose internal backend implementation details through IPC.

Treat IPC as a stable public API.

---

# Single Source of Truth

Business rules must exist in one place only.

Avoid duplicating logic between Rust and TypeScript.

If validation or calculations are required, prefer implementing them in Rust and exposing the results through IPC.

---

# Rust Guidelines

Prefer:

- ownership over unnecessary cloning

- explicit error handling

- Result<T, E>

- idiomatic Rust

- modular crates

- strong typing

Avoid:

- unwrap()

- expect() outside tests

- panic! for recoverable errors

- unnecessary Arc<Mutex<T>>

- global mutable state

Prefer immutable data structures whenever practical.

---

# Vue Guidelines

Use:

- Composition API

- script setup

- TypeScript

- composables

- reusable components

- PrimeVue v4 components (`Button`, `InputText`, `InputNumber`, `Select`, `ToggleSwitch`, etc.) over raw HTML elements or custom controls

- PrimeVue `v-tooltip` directive for all tooltips and hover hints (e.g., `v-tooltip.top="'...'"` or `v-tooltip="'...'"`) over native HTML `title` attributes

- centralized service wrappers (e.g., `@/services/nativeService`) for all native or system operations

Avoid:

- Options API

- large components

- business logic inside views

- direct IPC or Neutralino calls scattered across components

- direct imports of `@neutralinojs/lib` in components, views, or composables (must use `@/services/nativeService` or domain services)

- raw HTML inputs/buttons (`<button>`, `<input>`, `<select>`) or reinventions of controls already available in PrimeVue

- native HTML `title` attributes for tooltips or descriptive hints (always use PrimeVue `v-tooltip`)

Components should remain focused on rendering.

---

# Services

Frontend services should only wrap IPC calls.

Example flow:

View

↓

Composable

↓

Frontend Service (`@/services/*`, e.g., `@/services/nativeService`)

↓

Neutralino IPC / Native API (`@neutralinojs/lib`)

↓

Rust Backend

↓

Kubernetes

Components must never call IPC or `@neutralinojs/lib` directly. All native/OS/filesystem/IPC capabilities must be accessed via frontend services.

---

# State Management

Keep state local whenever possible.

Shared state should only exist when genuinely shared across multiple views.

Do not duplicate backend state inside multiple frontend stores.

---

# Configuration System

Application settings and user preferences are owned and persisted by the Rust backend.

## Architecture

- **Storage**: Persisted on disk as JSON in `~/.orbit/config.json`.
- **Logs Directory**: Application logs are stored in `~/.orbit/logs/`.
- **Backend Model (`core/engine/src/config.rs`)**:
  - Represented by `OrbitConfig`.
  - Serialized using `#[serde(rename_all = "camelCase")]` across the IPC boundary.
  - Supports backward compatibility aliases (`#[serde(alias = "...")]`) for legacy disk formats.
- **IPC Protocol**:
  - `getAppSettings`: Dispatched from frontend to request settings.
  - `updateAppSettings`: Dispatched from frontend with updated settings payload.
  - `appSettingsUpdated`: Event emitted by backend to broadcast updated configuration.
- **Frontend Layer**:
  - Strongly typed TS model in `@/types/settings`.
  - Service abstraction in `@/services/appSettingsService`.
  - Central reactive store in `@/stores/settingsStore`.

---

# Kubernetes

The Rust backend owns every interaction with Kubernetes.

Never access Kubernetes directly from the frontend.

Support:

- multiple clusters

- multiple contexts

- namespace switching

- watch APIs

- reconnect behavior

- authentication providers

Never assume:

- default namespace

- cloud provider

- Kubernetes version

---

# Security

Sensitive information never belongs in the frontend.

Do not expose:

- kubeconfig contents

- private keys

- bearer tokens

- certificates

- credentials

The frontend should only receive the minimum information necessary to render the UI.

---

# Error Handling

Errors returned through IPC should:

- include machine-readable error codes

- contain user-friendly messages

- avoid leaking implementation details

Unexpected failures should be logged by the backend.

---

# Performance

Prefer:

- streaming or incremental updates

- background workers

- caching

- lazy loading

- batching IPC requests

Avoid polling when Kubernetes watch APIs are available.

Avoid unnecessary serialization across IPC.

---

# Dependencies

Before adding dependencies:

Evaluate:

- maintenance

- security

- compile time

- binary size

- community adoption

Prefer standard library functionality whenever practical.

Avoid introducing dependencies for small utilities.

---

# Code Style

Write code that is easy to understand.

Prefer descriptive names.

Avoid abbreviations.

Good:

- clusterConnection

- activeNamespace

- workloadSummary

Avoid:

- ctx

- obj

- tmp

- data1

Variable Naming in CSS

Good:

- border-(--border)

- min-w-35

- shrink-0

Avoid

- border-[var(--border)]

- min-w-[140px]

- flex-shrink-0

---

# Comments

Comments should explain why.

Avoid comments that simply describe what the code is doing.

---

# Logging

Backend:

- log diagnostics

- log failures

- avoid sensitive data

Frontend:

- avoid excessive console logging

- never log secrets

---

# Documentation

Whenever a public IPC command changes:

Update:

- IPC documentation

- request schema

- response schema

- examples

Documentation should reflect implementation.

---

# Goal

Every contribution should move Orbit closer to being a professional-grade native Kubernetes desktop application with a clear separation between the Rust backend and the Vue frontend, connected through a stable, well-defined IPC interface.

---

# UI Design System

Orbit's visual language is ******technical, compact, information-dense, and monochrome/noir******. It must feel like a professional IDE or infrastructure console — not a consumer SaaS dashboard.

The canonical source of truth for all visual decisions is:

- `src/assets/base.css` — design tokens (colors, shadows, z-index, typography)

- `src/assets/main.css` — global base styles, scrollbar, page transitions

- `src/theme/orbitTheme.ts` — PrimeVue Nora preset customization

Before adding any CSS, always check these files first.

---

## UI Principles

Orbit's UI should feel like a professional IDE or infrastructure console rather than a consumer SaaS dashboard.

### Information Density

Prefer useful information over decorative whitespace while preserving scanability and clear hierarchy.

### Hierarchy

Establish hierarchy in this order:

1. Typography

2. Spacing

3. Surface contrast

4. Semantic color

5. Borders only when structural

Do not use every mechanism simultaneously for the same element.

### Progressive Disclosure

Show information needed for the current task first. Move secondary information into drawers, popovers, expandable sections, or contextual panels.

### Interaction Locality

Actions should appear near the resource or state they affect.

### Familiarity

Prefer interaction patterns established by VS Code, Kubernetes tooling, IDEs, and infrastructure consoles unless a different pattern clearly improves the workflow.

### Consistency

Before creating or modifying a UI component:

1. Search for existing instances of the same component or interaction.

2. Reuse the established implementation.

3. If the component is shared, modify the shared component instead of creating a local variant.

4. Do not introduce a new visual variant without a clear semantic reason.

The same interaction should look and behave consistently across Orbit.



---

## Typography

### Fonts

| Role      | Font family                                          | Token         |

|-----------|------------------------------------------------------|---------------|

| UI text   | Inter → Manrope → system-ui                          | `--font-ui`   |

| Monospace | Geist Mono → JetBrains Mono → Cascadia Code → Fira Code | `--font-mono` |

Use `font-ui` for all prose, labels, navigation, and controls.

Use `font-mono` for log output, YAML/JSON editors, resource names that must preserve exact casing, and any terminal-style output.

### Text Scale

| Usage                    | Size class      | Weight     |

|--------------------------|-----------------|------------|

| Page/section title       | `text-xl`       | `font-bold` |

| Card/drawer title        | `text-lg`       | `font-bold` |

| Table headers, labels    | `text-sm`       | `font-medium` or `font-semibold` |

| Table cell content       | `text-xs`       | `font-normal` or `font-medium` |

| Footer / status bar      | `text-[11px]`   | `font-medium` |

| Inline badges/tags       | `text-xs`       | `font-semibold` |

| Breadcrumb               | `text-xs`       | `font-medium` |

Never use type sizes above `text-xl` for data-density screens. Reserve large type only for empty states or welcome/onboarding screens.

### Tracking and Leading

- Use `tracking-tight` for headings and cluster names in the footer.

- Use `tracking-wider` for secondary metadata labels (uppercase status bar items).

- Default `line-height: 1.5` is set globally; do not override locally.

---

## Color

### Design Tokens

Use CSS custom properties from `base.css`. Never hard-code hex values that duplicate an existing token.

#### Brand / Accent

| Token            | Light                      | Dark                       |

|------------------|----------------------------|----------------------------|

| `--accent`       | `#4f8cff`                  | `#6aa8ff`                  |

| `--accent-hover` | `#6ca2ff`                  | `#84b8ff`                  |

| `--accent-active`| `#3c78e8`                  | `#4e95ff`                  |

| `--accent-soft`  | `rgba(79,140,255, 0.12)`   | `rgba(106,168,255, 0.14)`  |

#### Backgrounds

| Token          | Purpose                        |

|----------------|--------------------------------|

| `--bg-app`     | Root application background    |

| `--bg-sidebar` | Sidebar / activity bar         |

| `--bg-panel`   | Content panels                 |

| `--bg-card`    | Card surfaces                  |

| `--bg-hover`   | Hover state for interactive rows/items |

| `--bg-active`  | Pressed/active state           |

#### Text

| Token              | Usage                                   |

|--------------------|-----------------------------------------|

| `--text-primary`   | Default body and heading text           |

| `--text-secondary` | Supporting labels, descriptions         |

| `--text-muted`     | Deemphasized metadata, timestamps       |

| `--text-disabled`  | Disabled controls and unavailable items |

In Tailwind, these map to `text-primary`, `text-muted-color`, etc. via `tailwindcss-primeui`. Prefer these semantic classes over raw Tailwind gray shades.

#### Borders

| Token             | Usage                              |

|-------------------|------------------------------------|

| `--border`        | Default structural borders         |

| `--border-strong` | Emphasized separators, focus rings |

In Tailwind: `border-(--border)` and `border-(--border-strong)`.

#### Status / Semantic

| Token            | Meaning                    |

|------------------|----------------------------|

| `--success`      | Healthy, running, complete |

| `--warning`      | Pending, degraded, unknown |

| `--danger`       | Failed, error, crash       |

| `--info`         | Informational, neutral     |

Each status color has a paired `-soft` variant for background fills (e.g. `--success-soft`).

Do not invent new semantic colors. Map all states to one of the four above.

#### Kubernetes Resource Colors

Each Kubernetes resource kind has a dedicated color token used for icons, dots, and accents:

| Resource     | Token (CSS var)   | Tailwind class         |

|--------------|-------------------|------------------------|

| Deployment   | `--deployment`    | `text-deployment`      |

| DaemonSet    | `--daemonset`     | `text-daemonset`       |

| StatefulSet  | `--statefulset`   | `text-statefulset`     |

| Job          | `--job`           | `text-job`             |

| Pod          | `--pod`           | `text-pod`             |

| ReplicaSet   | `--replicaset`    | `text-replicaset`      |

| Node         | `--node`          | `text-node`            |

| Secret       | `--secret`        | `text-secret`          |

| ConfigMap    | `--configmap`     | `text-configmap`       |

| Service      | `--service`       | `text-service`         |

| Ingress      | `--ingress`       | `text-ingress`         |

Use these tokens consistently — never assign an arbitrary color to a resource kind.

### Color Usage Rules

- Use semantic colors (`--success`, `--danger`, etc.) for communicating state. Never use them decoratively.

- Accent color (`--accent`) is reserved for interactive focus, primary actions, and selected states. Do not scatter it as a general highlight.

- Text on dark backgrounds must use the dark-mode token variants. Do not invert manually.

- Do not introduce new color values that are not derived from an existing token or a Kubernetes resource.

---

## Spacing

 Orbit uses Tailwind's default spacing scale. The following values are standard across components:

 | Context                        | Value             |

 |--------------------------------|-------------------|

 | Main content area padding      | `p-8`             |

 | Card/panel inner padding       | controlled by PrimeVue Card |

 | Table toolbar gap              | `gap-4`           |

 | Control group gap              | `gap-2` / `gap-3` |

 | Inline icon + label gap        | `gap-1.5`         |

 | Section vertical gap           | `gap-8` / `gap-10` |

 | Footer / header horizontal pad | `px-3`            |

 | Footer / header vertical pad   | `py-1` (footer) / `py-2` (header) |

 | Drawer inner header margin     | `mb-2`            |

 Do not use arbitrary spacing values (`min-w-[140px]`, `gap-[18px]`). Prefer the nearest Tailwind scale step or an existing pattern.

 ---

 ## Component Grouping Without Borders

 When spacing alone is insufficient to distinguish complex component groups, use the following ******borderless grouping techniques****** inspired by the PrimeVue Nora theme:

 1. ******Proportional Ratio Spacing (1:3 Rhythm)******:

    - Intra-item gap (between label and input): ******tight****** (`gap-1.5` / `gap-2`).

    - Inter-item gap (between controls in the same sub-group): ******medium****** (`gap-3` / `gap-4`).

    - Inter-group gap (between major sections): ******wide****** (`gap-8` / `gap-10`).

 2. ******Subtle Surface Tone Shifts (Zonal Backgrounds)******:

    - Use flat, borderless background fills (`bg-(--bg-hover)/40`, `bg-surface-50`, `bg-surface-900`) with soft radii to visually group related sub-widgets or complex inputs without any stroke lines.

 3. ******Two-Column / Asymmetric Layout******:

    - Pair an informative left sidebar/header column (\~25–30% width: title, micro-description, icon) with interactive controls on the right column (\~70–75% width) to break reading flow and create natural section landmarks.

 4. ******Typographic Rhythm & Eyebrow Headers******:

    - Use small, uppercase tracked overlines (`text-xs font-semibold tracking-wider text-muted-color`) paired with contextual micro-descriptions to demarcate section boundaries clearly.

 5. ******Grouped Inset Wells******:

    - Group repeatable items (e.g. Key-Value pairs, Port lists, Environment variables) in borderless inset wells with unified inner padding.

 ---

 ## Shadows

 Shadows are used sparingly. They communicate layering — not decoration.

 | Token         | Usage                                |

 |---------------|--------------------------------------|

 | `--shadow-sm` | Subtle lift for inputs, small cards  |

 | `--shadow`    | Dropdowns, overlays, popovers        |

 Do not add `box-shadow` outside of these two tokens. Hierarchy is established through borders and background color contrast, not shadow depth.

 ---

 ## Z-Index Scale

 | Token          | Value | Layer                          |

 |----------------|-------|--------------------------------|

 | `--z-sticky`   | 100   | Sticky headers, toolbars       |

 | `--z-dropdown` | 1000  | Select/dropdown menus          |

 | `--z-overlay`  | 1030  | Sidebars, panels               |

 | `--z-modal`    | 1050  | Dialogs                        |

 | `--z-popover`  | 1060  | Popovers, column configurators |

 | `--z-tooltip`  | 1070  | Tooltips                       |

 | `--z-toast`    | 1080  | Toast notifications            |

 Always use a token. Never use a hard-coded z-index value.

 ---

 ## Layout

 ### Application Shell

 ```

 ┌────────────────────────────────────────────────────┐

 │  [Activity Bar] [Sidebar Panel] │ [Header]          │  ← shrink-0

 │                                 │ ─────────         │

 │                                 │ [Main Content]    │  ← flex-1 overflow-y-auto

 │                                 │                   │

 │                                 │ p-8 container     │

 ├────────────────────────────────────────────────────┤

 │  [Footer — status bar]                              │  ← shrink-0

 └────────────────────────────────────────────────────┘

 ```

 - The root is `flex flex-col h-screen w-screen overflow-hidden`.

 - Sidebar is an `<aside>` composed of a narrow activity bar and a contextual panel.

 - Header and footer use `shrink-0` to remain fixed height.

 - Main content uses `flex-1 overflow-y-auto`.

 ### View Layout

 Every page view uses `ViewLayout` which provides:

 - A `text-xl font-bold tracking-tight` `<h2>` for the page title.

 - A `flex flex-col gap-6` content wrapper.

 - An optional `actions` slot aligned to the title row.

 Do not replicate this pattern inline in a view. Always use `ViewLayout`.

 ---

 ## Borders

Borders are structural, not decorative.

Allowed:

- table/container boundaries

- drawer/header separation

- input/control boundaries provided by PrimeVue

- structural separators required by the layout

Avoid:

- borders around every section

- borders used only to group form fields

- decorative cards

- arbitrary `<hr>` separators

Use spacing, typography, and subtle surface changes for in-page grouping when a structural border is unnecessary.



---

## Borders and Radius

- Orbit's UI is inspired by the ******PrimeVue Nora theme******, emphasizing flatness, minimalism, and simplicity.

- Do NOT use borders, card wrappers, or `<hr>` lines to group in-page content or divide form sections. Use the borderless grouping techniques above instead.

- Data tables use a flat, single-container design (`border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)`) without elevated `<Card>` wrappers. Header rows use subtle zonal tone shifts (`bg-surface-100` light / `bg-surface-900` dark) with a crisp bottom separator, while data rows use subtle separators and responsive hover states (`hover\:bg-(--bg-hover)`).

- Input controls use `variant="filled"` or the PrimeVue Nora default radius (small, consistent) without heavy border chrome.

- Drawer title sections use `border-b border-(--border)`.

- Do not apply large or pill-shaped radii to data containers.

---

## PrimeVue Component Usage

Use PrimeVue v4 components for interactive controls when an equivalent component exists.

Examples:

- Button → `Button`

- Text input → `InputText`

- Number input → `InputNumber`

- Select → `Select`

- Toggle → `ToggleSwitch`

Do not recreate a PrimeVue control with custom HTML/CSS without a concrete reason.

Native semantic HTML remains appropriate where PrimeVue does not provide the relevant semantic element, such as links, tables, forms, labels, and structural elements.



---

## PrimeVue Theme

Orbit uses ******PrimeVue v4 with the Nora preset****** customized via `orbitTheme.ts`.

### Primary Palette

| Mode  | Primary color                | Hover              |

|-------|------------------------------|--------------------|

| Light | `zinc.950` (near black)      | `zinc.900`         |

| Dark  | `slate.50` (near white)      | `slate.100`        |

This produces a high-contrast, monochrome primary action color in both modes.

### Focus Ring

- Width: `2px`, style: `dashed`, color: `{primary.color}`, offset: `2px`.

- Do not override the focus ring locally. It is set globally.

### Highlight

- Light: `zinc.950` background with white text.

- Dark: `rgba(250,250,250,.16)` background with near-white text.

### Component Sizing

- Prefer `size="small"` for toolbar actions, icon buttons, and secondary controls.

- Prefer `variant="text"` + `severity="secondary"` for icon-only toolbar buttons.

- Use `fluid` on inputs inside filter toolbars.

---

## Status Indicators

Always use `StatusBadge` for Kubernetes resource phase/condition display.

| Status | Token |

|---|---|

| Running / Completed / Active | `--success` |

| Pending / Progressing | `--warning` |

| Failed / CrashLoopBackOff / Terminating | `--danger` |

| Unknown / Other | `--info` |

Do not use raw Tailwind color classes for Kubernetes status.

Status colors must come from the design tokens and must communicate semantic state, not decoration.



---

## Icons

Orbit uses ******Lucide Vue****** for all UI icons.

- Icon size in toolbars and table cells: `w-4 h-4` (`16px`).

- Icon size in the footer status bar: `:size="12"`.

- Icon size in drawer headers: `w-3.5 h-3.5`.

- Icon color in controls: `text-muted-color` (deemphasized) unless the icon is the primary affordance.

- Do not mix icon libraries. Do not use PrimeIcons for UI chrome (they are used by PrimeVue internally).

---

## Scrollbars

Custom scrollbar styles are set globally in `main.css`:

- Width/height: `6px`.

- Track: transparent.

- Thumb: `var(--border)`, radius `3px`.

- Thumb on hover: `var(--border-strong)`.

Do not override scrollbar styles per-component.

---

## Animations and Transitions

| Transition          | Duration   | Easing                          | Usage                        |

|---------------------|------------|---------------------------------|------------------------------|

| Page route change   | `0.2s`     | `cubic-bezier(0.4, 0, 0.2, 1)` | Fade + 4px vertical translate |

| Hover states        | Tailwind default (`transition-colors`) | — | Breadcrumb links, nav items |

Keep transitions subtle. Orbit is a tool — motion should confirm interaction, not entertain.

Do not introduce new animation keyframes without a concrete interaction justification.

---

## Dark Mode

Dark mode is applied via the `.my-app-dark` class on a root ancestor.

- All design tokens have dark variants defined in `base.css`.

- PrimeVue dark mode is handled by the Nora theme's `colorScheme.dark` configuration.

- Use Tailwind's `dark:` variant only for values that are not covered by the CSS tokens.

- Do not hard-code light-only colors in components. Always use token-based classes.

---

## UI Anti-Patterns

Never use any of the following:

- Raw `<button>`, `<input>`, `<select>` when an equivalent PrimeVue component exists.

- Hard-coded hex color values outside of `base.css`.

- Arbitrary spacing or sizing: `w-[140px]`, `gap-[18px]`, `text-[13px]` (use the nearest Tailwind step or an existing token).

- `border-[var(--border)]` instead of `border-(--border)`.

- `flex-shrink-0` instead of `shrink-0`.

- `min-w-[140px]` instead of `min-w-35`.

- Multiple local overrides of the same PrimeVue component selector.

- Shadow or border purely for decoration.

- Borders, card wrappers, or `<hr>` lines used to group content or divide form sections instead of using natural vertical spacing (`gap-6`, `gap-8`).

- Gradients for backgrounds or text.

- Color accents on UI elements that carry no semantic meaning.

- Rounded pill shapes on data containers or table rows.

- Duplicate `StatusBadge` logic inlined in a table column.

- Z-index values that are not a token.

---

# Definition of Done

A task is complete only when:

- The requested behavior is implemented.

- Existing architecture and conventions are followed.

- Existing reusable components and abstractions were considered.

- Existing patterns were evaluated for suitability rather than copied mechanically.

- Any new abstraction or design pattern has a concrete justification.

- No unnecessary files or speculative abstractions were introduced.

- Relevant tests pass.

- Type checking passes.

- Linting passes when configured.

- The affected application/package builds successfully when applicable.

- The final diff contains only task-related changes.

- UI changes follow the Orbit design system.

- No existing API was assumed without repository evidence.

- Verification results and remaining limitations are reported honestly.