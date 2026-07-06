# PR Review: #1 — feat: implement overview dashboard UI prototype and app layout

**Reviewed**: 2026-07-06
**Author**: @vantoan1511 (NGUYEN VAN TOAN)
**Branch**: `feat/overview-ui-prototype` → `main`
**Decision**: APPROVE

## Summary

This PR implements the dashboard landing page prototype (`DashboardView.vue`) along with standard structural layouts (`AppLayout.vue`, `AppHeader.vue`, `AppSidebar.vue`) and six dashboard widgets displaying statuses, usage, alerts/warnings, top resource consumers, and workloads. The implementation is clean, follows Vue 3 composition style, compiles successfully, passes TypeScript checks, and provides an excellent foundation for the Orbit Kubernetes application.

## Findings

### CRITICAL

None

### HIGH

None

### MEDIUM

1. **Hardcoded State & Mock Data**:
   - Files like `AppSidebar.vue`, `AppHeader.vue`, `HealthOverview.vue`, `ResourceUsage.vue`, `TopConsumers.vue`, `WorkloadSummaries.vue`, and `AlertsEvents.vue` contain mock datasets (e.g. cluster listings, resource metrics, event streams, counts). As the project progresses, these must be migrated to retrieve active states from the Rust backend via Neutralino IPC service boundaries, following the project guidelines in `GEMINI.md`.

2. **Inline Styling / Hardcoded Colors in Script Setup**:
   - `ResourceUsage.vue` makes use of canvas-level canvas gradient fills and text colors which rely on a check on `document.documentElement.classList.contains('my-app-dark')` on mount. This won't dynamically update if the theme changes post-mount unless handled reactively or rebuilt using Vue state observers.

### LOW

1. **Theme Toggle DOM Manipulation**:
   - `AppSidebar.vue` directly mutates `document.documentElement` class list and attributes. While functional, it is cleaner to encapsulate theme management in a shared composable or store.
2. **Standardized Anchor Links**:
   - In `AppSidebar.vue` line 143, the documentation button has `href="#"`. This should eventually point to real resources or call system APIs.

## Validation Results

| Check      | Result                         |
| ---------- | ------------------------------ |
| Type check | Pass                           |
| Lint       | Skipped (No script configured) |
| Tests      | Skipped (No script configured) |
| Build      | Pass                           |

## Files Reviewed

- `components.d.ts` (Modified)
- `neutralino.config.json` (Modified)
- `package-lock.json` (Modified)
- `package.json` (Modified)
- `src/App.vue` (Modified)
- `src/components/dashboard/AlertsEvents.vue` (Added)
- `src/components/dashboard/HealthOverview.vue` (Added)
- `src/components/dashboard/ResourceUsage.vue` (Added)
- `src/components/dashboard/StatusSummary.vue` (Added)
- `src/components/dashboard/TopConsumers.vue` (Added)
- `src/components/dashboard/WorkloadSummaries.vue` (Added)
- `src/components/layout/AppHeader.vue` (Added)
- `src/components/layout/AppLayout.vue` (Added)
- `src/components/layout/AppSidebar.vue` (Added)
- `src/router/index.ts` (Modified)
- `src/views/DashboardView.vue` (Added)
- `src/views/HomeView.vue` (Deleted)
