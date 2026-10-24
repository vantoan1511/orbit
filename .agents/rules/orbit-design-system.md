---
trigger: always_on
---

# Orbit Design System

## Authority

This rule is the visual constitution of Orbit.

Any UI implementation or modification MUST preserve these principles. Existing Orbit screens and the existing PrimeVue/Nora theme are the primary visual reference. Do not invent a new visual language for an individual screen.

## Product Character

Orbit is a fast desktop application for exploring, monitoring, and managing Kubernetes clusters.

The visual language is inspired by the **PrimeVue Nora theme**: flat, minimalist, and simple.

The intended visual language is:

- technical
- compact
- information-dense
- flat & minimalist (Nora theme foundation)
- restrained
- functional
- monochrome/noir
- desktop-oriented

Orbit should feel closer to a professional IDE, database client, or infrastructure console than a consumer SaaS dashboard.

Avoid:

- marketing/SaaS aesthetics
- excessive rounded cards
- borders or `<hr>` lines used to group content or divide form sections (use natural spacing instead)
- excessive whitespace
- gradients used as decoration
- decorative shadows
- oversized typography
- colorful UI without semantic meaning
- one-off visual patterns

## Layout

Prefer:

- persistent navigation
- workspace-oriented content
- compact toolbars
- dense data tables
- clear structural hierarchy
- natural whitespace and spacing to separate sections
- predictable alignment

The data/workspace should be the visual center. Do not wrap every section in a visually heavy card. Do not introduce `<hr>` divider lines or container borders to separate logical blocks within a form or panel; use clean vertical spacing (`gap-6` / `gap-8`) and typography instead.

## Spacing and Density

Optimize for scanning and frequent interaction by technical users.

Prefer natural, compact, consistent spacing over borders for grouping related items.

Do not arbitrarily change padding, height, line-height, or gaps for individual screens when an existing token or component pattern already exists.

## Component Grouping Without Borders

When spacing alone is insufficient to distinguish complex component groups, use the following **borderless grouping techniques** inspired by the PrimeVue Nora theme:

1. **Proportional Ratio Spacing (1:3 Rhythm)**:
   - Intra-item gap (between label and input): **tight** (`gap-1.5` / `gap-2`).
   - Inter-item gap (between controls in the same sub-group): **medium** (`gap-3` / `gap-4`).
   - Inter-group gap (between major sections): **wide** (`gap-8` / `gap-10`).
2. **Subtle Surface Tone Shifts (Zonal Backgrounds)**:
   - Use flat, borderless background fills (`bg-(--bg-hover)/40`, `bg-surface-50`, `bg-surface-900`) with soft radii to visually group related sub-widgets or complex inputs without any stroke lines.
3. **Two-Column / Asymmetric Layout**:
   - Pair an informative left sidebar/header column (~25–30% width: title, micro-description, icon) with interactive controls on the right column (~70–75% width) to break reading flow and create natural section landmarks.
4. **Typographic Rhythm & Eyebrow Headers**:
   - Use small, uppercase tracked overlines (`text-xs font-semibold tracking-wider text-muted-color`) paired with contextual micro-descriptions to demarcate section boundaries clearly.
5. **Grouped Inset Wells**:
   - Group repeatable items (e.g. Key-Value pairs, Port lists, Environment variables) in borderless inset wells with unified inner padding.

## Borders, Radius, and Elevation

Use borders sparingly and strictly for primary structural containers (e.g. data table outer frames, persistent activity bars, or drawer boundaries).

Data tables use a flat, single-container frame (`border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)`) rather than elevated `<Card>` components. Header rows use subtle zonal backgrounds with a bottom separator to anchor columns, data rows use soft row separators with active hover states (`bg-(--bg-hover)`), and numeric/metric columns should be strictly right-aligned to maintain scannability.

Do NOT use borders, card wrappers, or `<hr>` dividers to group in-page content or form sections. Use the borderless grouping techniques above instead.

Do not add borders to every nested component.

Use small, restrained radii. Avoid excessive pill-shaped controls unless the component semantics require them.

Avoid decorative shadows. Use contrast, spacing, and subtle layering to communicate hierarchy.

## Color

Use a restrained monochrome foundation.

Semantic colors are for communication:

- success
- warning
- danger/error
- information

Do not introduce accent colors merely to make a screen more visually interesting.

Use the existing PrimeVue Nora theme and Orbit design tokens before adding new values.

## Typography

Typography must prioritize:

1. hierarchy
2. readability
3. information density
4. consistency

Avoid oversized headings and consumer-app typography.

Do not change typography locally unless the existing design system does not provide the required hierarchy.

## PrimeVue Is the Component Foundation

Orbit uses PrimeVue v4 with the Nora theme.

PrimeVue components and the Nora theme are the default source of truth for:

- buttons
- inputs
- selects
- dropdowns
- tabs
- dialogs
- overlays
- menus
- toggles
- badges
- tooltips (`v-tooltip` directive)
- tables
- pagination
- forms
- other standard controls

Do not recreate a PrimeVue component with custom HTML/CSS when the PrimeVue component can satisfy the requirement.

When Orbit needs visual customization, customize the shared PrimeVue theme or shared component styling rather than creating page-specific variants.

## PrimeVue Customization Rule

A visual customization of a PrimeVue component is a SYSTEM decision, not a page decision.

If a PrimeVue component needs to look different:

1. Inspect the existing Nora/PrimeVue configuration.
2. Search the repository for existing customizations of that component.
3. Reuse the existing customization if it solves the requirement.
4. If the customization is broadly applicable, implement it centrally.
5. If it is truly contextual, keep the override local but do not create a competing visual variant.
6. If the same component is being customized in a second place, STOP and consolidate the styles before proceeding.

Never create two independent styles for the same PrimeVue component for the same semantic purpose.

## Pattern Before Invention

Before creating a new UI pattern:

1. Search the codebase for an existing equivalent.
2. Find at least one analogous Orbit screen.
3. Identify the existing component and styling pattern.
4. Reuse it when possible.
5. Extend it when appropriate.
6. Only create a new pattern when there is a concrete reason.

Convenience is not sufficient justification for a new visual pattern.

## Consistency Over Local Optimization

A locally "better" UI is not better if it introduces a second visual language.

When two reasonable implementations exist, prefer the one that:

- reuses existing PrimeVue components
- reuses existing tokens
- reuses existing spacing
- reuses existing component styling
- looks like neighboring Orbit screens
- creates fewer new concepts

## Definition of Done for UI

Before declaring UI work complete, verify:

- The screen looks like Orbit.
- PrimeVue/Nora remains the visual foundation.
- Existing components and patterns were searched before creating new ones.
- No duplicate PrimeVue customization was introduced.
- No unnecessary cards, borders, shadows, colors, or rounded elements were added.
- Density and spacing match nearby screens.
- Semantic colors are used consistently.
- The implementation is reusable where the pattern is reusable.
- The result does not create a new visual variant accidentally.

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


### Workspace Tab Bars & Document Tabs

For multi-tab workspace layouts (e.g. `AppTabBar.vue` and `AppTabItem.vue`):

- **Active Tab**:
  - Seamless flush integration with workspace content (`bg-(--bg-card)`, `border-b-(--bg-card)` or seamless bottom blending).
  - High-contrast typography (`text-primary font-medium text-xs`).
  - Semantic resource icon coloring (`metadata.iconColorClass`, e.g. `text-deployment`, `text-pod`).
  - Persistently visible close button (`opacity-100`) for direct one-click dismissal.
  - **Forbidden**: Decorative colored top borders or accent stripes (e.g. `border-t-(--accent)`). Tabs must adhere to the monochrome/noir IDE aesthetic.
- **Inactive Tab**:
  - Recessed into the tab bar background (`bg-(--bg-sidebar)/70`, `text-muted-color`, `border-b-(--border)`).
  - Subtle hover feedback (`hover:bg-(--bg-hover)/60 hover:text-primary`).
  - Close button hidden by default, visible only on hover (`opacity-0 group-hover:opacity-100`).

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

- Tooltips → `v-tooltip` directive (do not use native HTML `title` attributes)

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
