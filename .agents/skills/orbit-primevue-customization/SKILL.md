---
name: orbit-primevue-customization
description: Customizes PrimeVue v4 components in Orbit using the existing Nora theme and shared styling strategy. Use when changing PrimeVue component appearance, theme tokens, PassThrough configuration, DataTable density, forms, buttons, tabs, dialogs, overlays, or shared component styling.
---

# Orbit PrimeVue Customization

## Core Rule

PrimeVue component styling is shared application infrastructure.

Never treat a PrimeVue customization as an isolated page-level decision without first checking the existing canonical implementation.

## Workflow

### Step 1: Locate the Canonical Configuration

Search the repository for:

- PrimeVue initialization
- Nora preset
- `definePreset`
- `theme`
- `components`
- `semantic`
- `primitive`
- `pt`
- `dt`
- PrimeVue component selectors
- shared wrappers
- existing component styles

Read the relevant files before modifying anything.

### Step 2: Search Existing Customizations

Search for the exact component.

For example, for Button inspect:

- `Button`
- `.p-button`
- PassThrough Button configuration
- Button-related theme configuration
- shared Button wrappers

Do the same for every component being changed.

### Step 3: Determine Scope

Use this decision tree:

Global visual language:
-> PrimeVue/Nora theme or shared design token.

Shared Orbit behavior/pattern:
-> shared Orbit component.

Unique page layout:
-> page-local layout styling.

Truly exceptional one-off:
-> local styling, with an explicit reason.

### Step 4: Prefer Theme-Level Customization

Use the PrimeVue v4 theming system already established by the project.

Prefer shared tokens and component configuration over repeated CSS selectors.

Do not introduce a second theme/preset for a single screen.

### Step 5: Avoid Style Drift

Never do this:

```css
/* DeploymentView.vue */
.p-button {
  height: 34px;
}

/* ServiceView.vue */
.p-button {
  height: 32px;
}
```

If both represent the normal Orbit Button, there must be one canonical value.

Likewise, avoid:

```css
.p-inputtext { ... }
```

in multiple unrelated SFCs for the same visual purpose.

### Step 6: Check Impact

After changing shared PrimeVue styling:

- inspect usages
- identify screens likely affected
- verify at least one table-heavy screen and one form-heavy screen when applicable
- preserve intentional contextual differences

Do not blindly optimize for one screen at the expense of the application.

## When a New Variant Is Legitimate

A new variant is justified only when the semantic purpose is different.

Examples:

- primary action vs destructive action
- compact table action vs normal form action
- dense data-grid control vs normal page control

Even then, use the project's established variant mechanism rather than creating arbitrary CSS.

## Final Checklist

- [ ] Existing PrimeVue configuration inspected
- [ ] Nora configuration inspected
- [ ] Existing component customizations searched
- [ ] Correct scope chosen
- [ ] No duplicate shared CSS
- [ ] No accidental second visual variant
- [ ] Existing screens checked
- [ ] Shared styling remains the single source of truth
