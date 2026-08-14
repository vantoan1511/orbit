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

Every implementation should favor simplicity, predictability, and long-term maintainability.

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

Avoid:

- Options API
- large components
- business logic inside views
- direct IPC calls scattered across components
- raw HTML inputs/buttons (`<button>`, `<input>`, `<select>`) or reinventions of controls already available in PrimeVue

Components should remain focused on rendering.

---

# Services

Frontend services should only wrap IPC calls.

Example flow:

View

↓

Composable

↓

Frontend Service

↓

Neutralino IPC

↓

Rust Backend

↓

Kubernetes

Components should never call IPC directly.

---

# State Management

Keep state local whenever possible.

Shared state should only exist when genuinely shared across multiple views.

Do not duplicate backend state inside multiple frontend stores.

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

Orbit's visual language is **technical, compact, information-dense, and monochrome/noir**. It must feel like a professional IDE or infrastructure console — not a consumer SaaS dashboard.

The canonical source of truth for all visual decisions is:

- `src/assets/base.css` — design tokens (colors, shadows, z-index, typography)
- `src/assets/main.css` — global base styles, scrollbar, page transitions
- `src/theme/orbitTheme.ts` — PrimeVue Nora preset customization

Before adding any CSS, always check these files first.

---

## Typography

### Fonts

| Role      | Font family                                          | Token         |
|-----------|------------------------------------------------------|---------------|
| UI text   | Inter → Manrope → system-ui                          | `--font-ui`   |
| Monospace | Geist Mono → JetBrains Mono → Cascadia Code → Fira Code | `--font-mono` |

Use `font-ui` for all prose, labels, navigation, and controls.
Use `font-mono` for log output, YAML/JSON editors, resource names that must preserve exact casing, and any terminal-style output.

### Text Scale

| Usage                    | Size class      | Weight     |
|--------------------------|-----------------|------------|
| Page/section title       | `text-xl`       | `font-bold` |
| Card/drawer title        | `text-lg`       | `font-bold` |
| Table headers, labels    | `text-sm`       | `font-medium` or `font-semibold` |
| Table cell content       | `text-xs`       | `font-normal` or `font-medium` |
| Footer / status bar      | `text-[11px]`   | `font-medium` |
| Inline badges/tags       | `text-xs`       | `font-semibold` |
| Breadcrumb               | `text-xs`       | `font-medium` |

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

| Token            | Light                      | Dark                       |
|------------------|----------------------------|----------------------------|
| `--accent`       | `#4f8cff`                  | `#6aa8ff`                  |
| `--accent-hover` | `#6ca2ff`                  | `#84b8ff`                  |
| `--accent-active`| `#3c78e8`                  | `#4e95ff`                  |
| `--accent-soft`  | `rgba(79,140,255, 0.12)`   | `rgba(106,168,255, 0.14)`  |

#### Backgrounds

| Token          | Purpose                        |
|----------------|--------------------------------|
| `--bg-app`     | Root application background    |
| `--bg-sidebar` | Sidebar / activity bar         |
| `--bg-panel`   | Content panels                 |
| `--bg-card`    | Card surfaces                  |
| `--bg-hover`   | Hover state for interactive rows/items |
| `--bg-active`  | Pressed/active state           |

#### Text

| Token              | Usage                                   |
|--------------------|-----------------------------------------|
| `--text-primary`   | Default body and heading text           |
| `--text-secondary` | Supporting labels, descriptions         |
| `--text-muted`     | Deemphasized metadata, timestamps       |
| `--text-disabled`  | Disabled controls and unavailable items |

In Tailwind, these map to `text-primary`, `text-muted-color`, etc. via `tailwindcss-primeui`. Prefer these semantic classes over raw Tailwind gray shades.

#### Borders

| Token             | Usage                              |
|-------------------|------------------------------------|
| `--border`        | Default structural borders         |
| `--border-strong` | Emphasized separators, focus rings |

In Tailwind: `border-(--border)` and `border-(--border-strong)`.

#### Status / Semantic

| Token            | Meaning                    |
|------------------|----------------------------|
| `--success`      | Healthy, running, complete |
| `--warning`      | Pending, degraded, unknown |
| `--danger`       | Failed, error, crash       |
| `--info`         | Informational, neutral     |

Each status color has a paired `-soft` variant for background fills (e.g. `--success-soft`).

Do not invent new semantic colors. Map all states to one of the four above.

#### Kubernetes Resource Colors

Each Kubernetes resource kind has a dedicated color token used for icons, dots, and accents:

| Resource     | Token (CSS var)   | Tailwind class         |
|--------------|-------------------|------------------------|
| Deployment   | `--deployment`    | `text-deployment`      |
| DaemonSet    | `--daemonset`     | `text-daemonset`       |
| StatefulSet  | `--statefulset`   | `text-statefulset`     |
| Job          | `--job`           | `text-job`             |
| Pod          | `--pod`           | `text-pod`             |
| ReplicaSet   | `--replicaset`    | `text-replicaset`      |
| Node         | `--node`          | `text-node`            |
| Secret       | `--secret`        | `text-secret`          |
| ConfigMap    | `--configmap`     | `text-configmap`       |
| Service      | `--service`       | `text-service`         |
| Ingress      | `--ingress`       | `text-ingress`         |

Use these tokens consistently — never assign an arbitrary color to a resource kind.

### Color Usage Rules

- Use semantic colors (`--success`, `--danger`, etc.) for communicating state. Never use them decoratively.
- Accent color (`--accent`) is reserved for interactive focus, primary actions, and selected states. Do not scatter it as a general highlight.
- Text on dark backgrounds must use the dark-mode token variants. Do not invert manually.
- Do not introduce new color values that are not derived from an existing token or a Kubernetes resource.

---

## Spacing

Orbit uses Tailwind's default spacing scale. The following values are standard across components:

| Context                        | Value             |
|--------------------------------|-------------------|
| Main content area padding      | `p-8`             |
| Card/panel inner padding       | controlled by PrimeVue Card |
| Table toolbar gap              | `gap-4`           |
| Control group gap              | `gap-2` / `gap-3` |
| Inline icon + label gap        | `gap-1.5`         |
| Section vertical gap           | `gap-6`           |
| Footer / header horizontal pad | `px-3`            |
| Footer / header vertical pad   | `py-1` (footer) / `py-2` (header) |
| Drawer inner header margin     | `mb-2`            |

Do not use arbitrary spacing values (`min-w-[140px]`, `gap-[18px]`). Prefer the nearest Tailwind scale step or an existing pattern.

---

## Shadows

Shadows are used sparingly. They communicate layering — not decoration.

| Token         | Usage                                |
|---------------|--------------------------------------|
| `--shadow-sm` | Subtle lift for inputs, small cards  |
| `--shadow`    | Dropdowns, overlays, popovers        |

Do not add `box-shadow` outside of these two tokens. Hierarchy is established through borders and background color contrast, not shadow depth.

---

## Z-Index Scale

| Token          | Value | Layer                          |
|----------------|-------|--------------------------------|
| `--z-sticky`   | 100   | Sticky headers, toolbars       |
| `--z-dropdown` | 1000  | Select/dropdown menus          |
| `--z-overlay`  | 1030  | Sidebars, panels               |
| `--z-modal`    | 1050  | Dialogs                        |
| `--z-popover`  | 1060  | Popovers, column configurators |
| `--z-tooltip`  | 1070  | Tooltips                       |
| `--z-toast`    | 1080  | Toast notifications            |

Always use a token. Never use a hard-coded z-index value.

---

## Layout

### Application Shell

```
┌────────────────────────────────────────────────────┐
│  [Activity Bar] [Sidebar Panel] │ [Header]          │  ← shrink-0
│                                 │ ─────────         │
│                                 │ [Main Content]    │  ← flex-1 overflow-y-auto
│                                 │                   │
│                                 │ p-8 container     │
├────────────────────────────────────────────────────┤
│  [Footer — status bar]                              │  ← shrink-0
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

## Borders and Radius

- Structural borders use `border-(--border)` or `border-surface-200/60` (via PrimeVue surface tokens).
- Table containers use `border border-surface rounded-lg overflow-hidden`.
- Drawer title sections use `border-b border-(--border)`.
- Input controls use the PrimeVue Nora default radius (small, consistent).
- Do not apply large or pill-shaped radii to data containers.

---

## PrimeVue Theme

Orbit uses **PrimeVue v4 with the Nora preset** customized via `orbitTheme.ts`.

### Primary Palette

| Mode  | Primary color                | Hover              |
|-------|------------------------------|--------------------|
| Light | `zinc.950` (near black)      | `zinc.900`         |
| Dark  | `slate.50` (near white)      | `slate.100`        |

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

Status is always communicated through `StatusBadge`:

| Status                              | Color   | Tailwind dot/text        |
|-------------------------------------|---------|--------------------------|
| Running / Completed / Active        | emerald | `bg-emerald-500` / `text-emerald-500` |
| Pending / Progressing               | amber   | `bg-amber-500` / `text-amber-500`     |
| Failed / CrashLoopBackOff / Terminating | rose | `bg-rose-500` / `text-rose-500`      |
| Unknown / Other                     | gray    | `bg-gray-400` / `text-gray-400`       |

Always use `StatusBadge` for Kubernetes resource phase/condition display. Do not inline status coloring in tables.

---

## Icons

Orbit uses **Lucide Vue** for all UI icons.

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

| Transition          | Duration   | Easing                          | Usage                        |
|---------------------|------------|---------------------------------|------------------------------|
| Page route change   | `0.2s`     | `cubic-bezier(0.4, 0, 0.2, 1)` | Fade + 4px vertical translate |
| Hover states        | Tailwind default (`transition-colors`) | — | Breadcrumb links, nav items |

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
- Gradients for backgrounds or text.
- Color accents on UI elements that carry no semantic meaning.
- Rounded pill shapes on data containers or table rows.
- Duplicate `StatusBadge` logic inlined in a table column.
- Z-index values that are not a token.
