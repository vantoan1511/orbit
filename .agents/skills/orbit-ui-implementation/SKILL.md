---
name: orbit-ui-implementation
description: Implements or modifies Orbit Vue UI using PrimeVue v4 and the Nora theme while preserving shared component patterns, visual consistency, and the Orbit technical desktop design. Use for UI screens, components, layouts, forms, tables, dialogs, navigation, and styling changes.
---

# Orbit UI Implementation

## Goal

Build UI that looks like it belongs to the existing Orbit application, not merely UI that satisfies the requested feature.

The Orbit Design System and existing code are the source of truth.

## Required Workflow

### 1. Understand Before Editing

Before coding:

- inspect the target page/component
- inspect the current PrimeVue configuration
- inspect the Nora theme/preset configuration
- inspect existing shared UI components
- find at least one analogous existing screen
- search for existing styling of every PrimeVue component you will customize

Do not start by creating new CSS or a new component.

### 2. Identify the Reuse Path

For each visual element, classify it:

- existing Orbit component -> reuse
- existing PrimeVue component -> reuse
- existing pattern that needs a small extension -> extend
- genuinely new interaction -> create a reusable component/pattern if appropriate

### 3. Decide Styling Scope

For every customization ask:

> Is this a global PrimeVue design decision, a shared Orbit component decision, or genuinely page-local?

Use this order:

1. PrimeVue/Nora theme
2. shared Orbit component
3. shared CSS/token
4. page-local styling

The narrower the scope, the stronger the justification required.

### 4. Implement

Prefer composition over duplication.

Avoid:

- copied CSS
- duplicate PrimeVue overrides
- page-specific component variants
- hard-coded repeated dimensions
- unnecessary wrapper components
- replacing PrimeVue components with custom markup

### 5. Consistency Check

Before finishing, search for:

- the PrimeVue component name
- its existing CSS selectors
- existing PassThrough configuration
- existing wrappers
- similar pages

If your implementation creates a second way of styling the same component, consolidate it.

### 6. Shift-Left Pre-Flight Scanner

Run the automated pre-flight scanner on modified files:

```bash
python .agents/scripts/verify_shift_left.py --changed
```

Ensure:
- Zero raw HTML `<button>`, `<input>`, `<select>` tags in `.vue` templates.
- Zero native HTML `title="..."` attributes (must use PrimeVue `v-tooltip`).
- Zero hardcoded hex colors in styles (must use tokens from `src/assets/base.css`).
- Zero direct `@neutralinojs/lib` imports in components or views.

## Visual Review

Check:

- density
- spacing
- typography
- border usage
- corner radius
- semantic colors
- table density
- control height
- sidebar/layout proportions
- hover/focus/disabled states

Compare against existing Orbit screens.

## Completion Criteria

The implementation is complete only when:

- behavior works
- existing patterns were reused
- PrimeVue/Nora remains the foundation
- styling is centralized at the correct scope
- no duplicate component styling was introduced
- the screen visually belongs to Orbit
