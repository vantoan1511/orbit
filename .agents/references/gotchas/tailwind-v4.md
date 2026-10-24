# Gotchas: Tailwind CSS v4 in Orbit

Orbit uses **Tailwind CSS v4** (`@tailwindcss/vite` v4.3.2). Several syntax conventions differ radically from Tailwind v3.

---

## 1. Important Modifier Exclamation Mark at the END (`p-0!` not `!p-0`)
- **Tailwind v3 (OLD/WRONG)**: `!p-0`, `!m-2`, `!bg-red-500`
- **Tailwind v4 (NEW/ORBIT)**: `p-0!`, `m-2!`, `bg-red-500!`
- **Why**: In Tailwind v4, exclamation mark position was moved to the trailing end to eliminate ambiguity with negation and CSS variables.

---

## 2. CSS Variable Utilities: Parentheses Syntax
- **Tailwind v3 (OLD/WRONG)**: `border-[var(--border)]`, `bg-[var(--bg-card)]`
- **Tailwind v4 (NEW/ORBIT)**: `border-(--border)`, `bg-(--bg-card)`, `text-(--text-primary)`
- **Rule in GEMINI.md**:
  - Good: `border-(--border)`, `min-w-35`, `shrink-0`
  - Avoid: `border-[var(--border)]`, `min-w-[140px]`, `flex-shrink-0`

---

## 3. Opacity Modifiers with CSS Variables
- To add alpha/opacity to a CSS variable in Tailwind v4:
  - `bg-(--bg-hover)/30`
  - `border-(--border)/40`
  - `text-(--text-muted)/80`
- Do NOT use arbitrary hex color opacity (`bg-[#ffffff10]`). Always use canonical variables.

---

## 4. No `@apply` with Complex Utility Chains
- In CSS blocks, prefer using standard CSS properties referencing tokens (`background: var(--bg-card);`) over long `@apply` chains.
- Do not define custom class names when PrimeVue `pt` (PassThrough) or standard Tailwind utility classes suffice.
