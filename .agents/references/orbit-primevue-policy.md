# Orbit PrimeVue Policy Reference

This document is a concise reference for agents. The actual PrimeVue/Nora configuration in the repository is authoritative.

## Stack

- Vue
- PrimeVue v4
- Nora theme
- Tauri v2 desktop application

## Canonical Styling Hierarchy

1. PrimeVue Nora theme / preset
2. PrimeVue component configuration
3. Orbit shared components
4. Orbit shared tokens/utilities
5. local component layout styles

A lower layer must not recreate a concern already owned by a higher layer.

## Common Ownership

| Concern | Preferred owner |
|---|---|
| Button visual treatment | Nora/PrimeVue shared theme |
| Input visual treatment | Nora/PrimeVue shared theme |
| Dropdown/Select visual treatment | Nora/PrimeVue shared theme |
| Tabs | Nora/PrimeVue shared theme |
| Dialog | Nora/PrimeVue shared theme |
| DataTable density | shared PrimeVue configuration/component pattern |
| Paginator | shared PrimeVue configuration/component pattern |
| Status semantics | shared Orbit pattern |
| Page spacing | shared layout/page pattern |
| Grid/flex positioning | local component/page |
| Resource-specific columns | resource page |
| Resource-specific data logic | resource page |

## Important Rule

If two screens need the same visual customization, there should be one shared implementation.

Do not copy the style into both screens.

## What Agents Should Search For

Before changing a PrimeVue component, search for:

- PrimeVue app setup
- Nora preset
- `definePreset`
- `theme`
- `components`
- `pt`
- `dt`
- `.p-`
- component name
- shared wrappers
- existing tokens

## Visual Baseline

Orbit should remain:

- compact
- flat
- restrained
- monochrome/noir
- technical
- information-dense
- desktop-oriented

The existing application is the visual reference. New UI should converge toward it, not diverge from it.
