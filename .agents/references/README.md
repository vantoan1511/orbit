# Orbit Curated Knowledge Base & Reference Catalog

This directory contains on-demand architectural decisions (ADRs), canonical implementation recipes, and domain gotchas for Orbit.

> **Progressive Disclosure Principle**:
> Do not read all files at once. Read this index first, then inspect only the specific document relevant to your task.

---

## Catalog Index

### 1. Architecture Decision Records (`architecture/`)
Deep rationale explaining **why** the system is built this way.
- [adr-001-application-lifecycle.md](file:///d:/Projects/orbit/.agents/references/architecture/adr-001-application-lifecycle.md) — Synchronous `app.mount()` before `init()`, IPC event handshake, and async store hydration.
- [adr-002-configuration-system.md](file:///d:/Projects/orbit/.agents/references/architecture/adr-002-configuration-system.md) — Backend-controlled settings model, `~/.orbit/config.json`, and dynamic UI rendering.
- [adr-003-storage-and-state.md](file:///d:/Projects/orbit/.agents/references/architecture/adr-003-storage-and-state.md) — Storage tiers: Native desktop storage vs Pinia reactive memory vs browser `localStorage` prohibition.

### 2. Canonical Recipes & Starter Templates (`recipes/`)
The authoritative "Gold Standard" patterns to mirror when adding new code.
- [canonical-resource-page.md](file:///d:/Projects/orbit/.agents/references/recipes/canonical-resource-page.md) — How to add a Kubernetes resource view (View, DataTable, and DetailsDrawer).
- [canonical-primevue-controls.md](file:///d:/Projects/orbit/.agents/references/recipes/canonical-primevue-controls.md) — PrimeVue v4 / Nora controls, forms, modal dialogs, and `v-tooltip` directives.
- [canonical-node-test.md](file:///d:/Projects/orbit/.agents/references/recipes/canonical-node-test.md) — Node 24 native test runner (`node:test`, `node:assert/strict`) starter templates.

### 3. Hard-Earned Gotchas & Troubleshooting (`gotchas/`)
Domain traps and non-obvious syntax quirks discovered during development.
- [tailwind-v4.md](file:///d:/Projects/orbit/.agents/references/gotchas/tailwind-v4.md) — Tailwind v4 conventions: trailing exclamation marks (`p-0!`), CSS variable utilities `border-(--border)`.
- [primevue-quirks.md](file:///d:/Projects/orbit/.agents/references/gotchas/primevue-quirks.md) — Tooltip directives vs native title, flat DataTable rules, PassThrough scope.
- [tauri-runtime.md](file:///d:/Projects/orbit/.agents/references/gotchas/tauri-runtime.md) — Prohibiting direct `@tauri-apps/api` imports in UI, storage timeouts, process lifecycle.
- [updater-backward-compat.md](file:///d:/Projects/orbit/.agents/references/gotchas/updater-backward-compat.md) — Compatibility zip bridge for runtime transitions (Neutralino to Tauri).

---

## How This Knowledge Base is Maintained
1. **Self-Improving via `/implement`**: When an unexpected deviation or gotcha is solved during implementation, the agent records it into the corresponding `gotchas/` file.
2. **Curated from User Corrections**: Repeated user corrections in `.agents/logs/` are synthesized into these reference docs using `curate_knowledge.py`.
3. **Repository Evidence Over Doc Text**: If a doc and the source code disagree, the source code and tests are authoritative. Docs capture invariants and rationale.
