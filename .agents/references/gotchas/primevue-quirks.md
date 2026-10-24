# Gotchas: PrimeVue v4 & Nora Theming Quirks

---

## 1. Never Use Native HTML `title` Attributes
- **Symptom**: Unstyled, delayed default browser tooltip pops up over sleek Nora dark UI.
- **Rule**: Use PrimeVue directive `v-tooltip="'...'"` or `v-tooltip.top="'...'"` on interactive components.
- **Gotcha**: On custom components like `BaseResourceDrawer`, `KeyValueEditor`, and `KeyValueBadgeList`, `title="..."` is a component prop, NOT a tooltip. `verify_shift_left.py` distinguishes between them.

---

## 2. PrimeVue `v-model` vs `model-value`
- PrimeVue v4 components use `v-model` (e.g. `v-model="val"`).
- In child custom editors, always emit `@update:model-value="handleUpdate"` when wrapping or forwarding form values.

---

## 3. Flat DataTable Overrides (No Outer Borders/Cards)
- In Orbit, `DataTable` must remain flat and borderless.
- Do NOT wrap `DataTable` in a `<Card>` component or add custom shadow boxes.
- Preserve the Nora density: row heights are compact (`size="small"`).

---

## 4. PassThrough (`pt`) Scope Discipline
- Only use component `pt` (PassThrough) when the styling is strictly contextual to that one screen.
- If the same `pt` override appears in 2 or more files, it belongs in the shared Nora preset configuration ([src/theme/orbitTheme.ts](file:///d:/Projects/orbit/src/theme/orbitTheme.ts)).
