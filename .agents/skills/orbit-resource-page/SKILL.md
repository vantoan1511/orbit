---
name: orbit-resource-page
description: Builds Kubernetes resource pages in Orbit using established resource-list patterns, PrimeVue v4 Nora components, shared tables, filters, pagination, status indicators, and contextual actions. Use when adding or modifying resource views such as Deployments, Pods, Services, ConfigMaps, Secrets, Jobs, or similar pages.
---

# Orbit Resource Page

## Goal

Create new Kubernetes resource pages by extending the existing Orbit resource-page pattern instead of inventing a new page layout.

## Required Workflow

1. Find the closest existing resource page.
2. Identify its shared layout, toolbar, table, filters, pagination, status, and action patterns.
3. Reuse those patterns.
4. Only introduce resource-specific behavior where Kubernetes semantics require it.

## Page Structure

Prefer the established Orbit hierarchy:

- page title
- resource navigation when applicable
- compact toolbar/filter area
- data table
- pagination
- contextual actions

Do not create a new card/container hierarchy for each resource type.

## Table

Use the existing PrimeVue DataTable pattern with flat, minimalist styling.

Preserve:

- flat & borderless hierarchy (no `<Card>` wrappers, no horizontal row divider lines)
- density and uniform row heights
- subtle zonal header styling with crisp bottom separator
- right-alignment for all numeric / metrics columns (CPU, Memory, Counts)
- sorting behavior
- pagination with subtle controls
- loading skeleton state
- empty state
- contextual action placement

Do not independently style or add custom card wrappers to the DataTable on each resource page.

## Filters

Reuse existing filter controls and spacing.

If a new filter is needed:

- use the existing PrimeVue component with `variant="filled"` for flush, borderless integration
- follow existing control sizing (`text-sm`, `min-w-40`)
- do not introduce a new filter visual style

## Status

Reuse existing semantic status presentation.

Do not invent a different color, badge, dot, or typography treatment for the same Kubernetes state.

## Actions

Reuse the existing contextual action/menu pattern.

Do not create a different action-button style simply because the resource has different operations.

## Completion

Verify the new page against at least one existing resource page before finishing.
