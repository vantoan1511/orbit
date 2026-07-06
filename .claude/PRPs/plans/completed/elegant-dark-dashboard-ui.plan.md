# Plan: Elegant Dark Dashboard UI

## Summary

Improve the dashboard UI to match the provided "elegant dark" mockup. This involves deepening the dark theme colors in `base.css` for a more premium feel, refactoring CSS variable usage across Vue components for Tailwind v4 consistency, and fixing hardcoded colors (like progress bars) to use theme variables.

## User Story

As a user, I want the dashboard to have a cohesive, premium dark theme matching the mockup, so that the application feels modern, elegant, and native to my operating system.

## Problem → Solution

**Current state**: The dark mode colors are slightly washed out, lacking the deep contrast of an "elegant dark" theme. Components inconsistently use CSS variables (mixing `bg-[var(--bg-card)]` with `bg-(--bg-card)`). Progress bars in `TopConsumers.vue` use hardcoded `bg-blue-500`.
**Desired state**: A deep, high-contrast dark theme with consistent Tailwind v4 variable syntax across all components. Dynamic colors applied correctly (e.g., CPU = blue, Memory = purple).

## Metadata

- **Complexity**: Medium
- **Source PRD**: N/A
- **PRD Phase**: N/A
- **Estimated Files**: ~5-8 files (base.css, AppHeader.vue, AppSidebar.vue, TopConsumers.vue, ResourceUsage.vue, etc.)

---

## UX Design

### Before

Dashboard has a standard gray-toned dark mode. Component classes have mixed Tailwind 3/4 syntaxes. Hardcoded blue progress bars regardless of the active metric (CPU vs Memory).

### After

Dashboard uses a near-black background with subtle, elevated cards. Component classes are uniformly using Tailwind v4 syntax (e.g. `text-(--text-primary)`). Progress bars adapt their color based on the selected metric.

### Interaction Changes

| Touchpoint                  | Before               | After                                                  | Notes                   |
| --------------------------- | -------------------- | ------------------------------------------------------ | ----------------------- |
| Top Consumers Progress Bars | Always `bg-blue-500` | Changes color based on tab (CPU: blue, Memory: purple) | Enhances visual context |

---

## Mandatory Reading

| Priority       | File                                        | Lines | Why                                                                |
| -------------- | ------------------------------------------- | ----- | ------------------------------------------------------------------ |
| P0 (critical)  | `src/assets/base.css`                       | All   | Contains the theme definitions to be refactored                    |
| P1 (important) | `src/components/layout/AppSidebar.vue`      | All   | Example of correct Tailwind v4 variable syntax `bg-(--bg-sidebar)` |
| P2 (reference) | `src/components/dashboard/TopConsumers.vue` | All   | Contains inconsistent syntax and hardcoded values                  |

## External Documentation

| Topic                     | Source        | Key Takeaway                                                         |
| ------------------------- | ------------- | -------------------------------------------------------------------- |
| Tailwind v4 CSS Variables | Tailwind Docs | Use `color-(--var-name)` syntax instead of `color-[var(--var-name)]` |

---

## Patterns to Mirror

### TAILWIND_V4_VARIABLE_SYNTAX

// SOURCE: src/components/layout/AppSidebar.vue:58

```html
<aside class="w-64 bg-(--bg-sidebar) border-r border-(--border) text-(--text-primary)"></aside>
```

---

## Files to Change

| File                                         | Action | Justification                                                          |
| -------------------------------------------- | ------ | ---------------------------------------------------------------------- |
| `src/assets/base.css`                        | UPDATE | Deepen dark theme colors, refine borders.                              |
| `src/components/dashboard/TopConsumers.vue`  | UPDATE | Refactor to use Tailwind v4 syntax, fix hardcoded progress bar colors. |
| `src/components/dashboard/ResourceUsage.vue` | UPDATE | Refactor to use Tailwind v4 syntax `bg-(--bg-card)`.                   |
| `src/components/layout/AppHeader.vue`        | UPDATE | Ensure consistent Tailwind v4 syntax.                                  |
| `src/components/layout/AppLayout.vue`        | UPDATE | Check and update background/text classes.                              |

## NOT Building

- New dashboard widgets not in the mockup.
- Changes to the underlying data fetching logic.
- Light mode redesign (only focusing on the dark theme right now).

---

## Step-by-Step Tasks

### Task 1: Refactor `base.css` Dark Theme Colors

- **ACTION**: Deepen the color palette in `base.css` under `[data-theme="dark"]`.
- **IMPLEMENT**: Update variables to match an "elegant dark" look (e.g., `--bg-app: #0B0C0F`, `--bg-sidebar: #111214`, `--bg-card: #16181B`, `--bg-panel: #16181B`, `--border: #262930`, `--border-strong: #3A3F4A`).
- **MIRROR**: N/A (CSS definition).
- **GOTCHA**: Ensure text colors maintain WCAG accessibility contrast against the darker backgrounds.
- **VALIDATE**: Inspect the app background and cards in the browser.

### Task 2: Standardize Tailwind v4 Syntax in Components

- **ACTION**: Replace `-[var(--variable)]` with `-(--variable)` across dashboard components.
- **IMPLEMENT**: Search and replace `bg-[var(--bg-card)]` with `bg-(--bg-card)`, `text-[var(--text-primary)]` with `text-(--text-primary)`, `border-[var(--border)]` with `border-(--border)`. Apply to `TopConsumers.vue`, `ResourceUsage.vue`, etc.
- **MIRROR**: TAILWIND_V4_VARIABLE_SYNTAX
- **VALIDATE**: Ensure styles are still properly applied after the syntax change.

### Task 3: Fix Hardcoded Colors in `TopConsumers.vue`

- **ACTION**: Make progress bars use theme colors based on the active tab.
- **IMPLEMENT**: Remove `color: "bg-blue-500"` from the mock data. In the template, dynamically apply classes: CPU uses `bg-(--accent)`, Memory uses `bg-(--namespace)` (purple), Restarts uses `bg-(--warning)` (yellow/orange).
- **GOTCHA**: Ensure the gradient widths in the progress bars correctly reflect the values.
- **VALIDATE**: Toggle between CPU, Memory, and Restarts tabs; verify progress bar colors change appropriately.

---

## Testing Strategy

### Edge Cases Checklist

- [ ] Theme toggling to Light Mode still works and doesn't break due to syntax changes.
- [ ] Contrast is sufficient for all text elements.
- [ ] Progress bars display correctly at 0% and 100%.

---

## Validation Commands

### Static Analysis

```bash
npm run type-check
```

EXPECT: Zero type errors

### Browser Validation

```bash
npm run dev
```

EXPECT: Feature works as designed, dark theme matches mockup, no styling regressions.

---

## Acceptance Criteria

- [ ] `base.css` dark theme variables updated to deeper, elegant dark tones.
- [ ] Component templates consistently use Tailwind v4 CSS variable syntax `bg-(--variable)`.
- [ ] Progress bars in `TopConsumers.vue` are dynamic and no longer hardcoded to blue.
- [ ] UI visually matches the provided `mockups/dashboard_view.png`.

## Completion Checklist

- [ ] Code follows discovered patterns.
- [ ] No hardcoded values (colors).
- [ ] No unnecessary scope additions.
- [ ] Self-contained — no questions needed during implementation.

## Risks

| Risk                               | Likelihood | Impact | Mitigation                                        |
| ---------------------------------- | ---------- | ------ | ------------------------------------------------- |
| CSS syntax updates break styles    | Low        | High   | Use browser dev tools to verify applied classes.  |
| Contrast issues with new darker bg | Medium     | Medium | Test carefully, keep `--text-muted` light enough. |

## Notes

The mockup suggests a seamless dark feel. Removing any unnecessary horizontal borders (if they conflict with the deep dark backgrounds) might also be required during implementation to fully match the mockup's cleanliness.
