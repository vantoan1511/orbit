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

---

## 5. No `size="normal"`
- PrimeVue components do not have a `"normal"` size prop. Standard tier is represented by omitting the `size` prop entirely (e.g. `<Button>`, `<InputText>`). Setting `size="normal"` is a TypeScript type error.

---

## 6. Custom Tokens Need a `css` Hook to Take Effect
- Defining custom properties (e.g. `backdropBlur`, `sheenHighlight`) via module augmentation on PrimeVue token sections generates CSS variables (`--p-<component>-...`), but PrimeVue's component stylesheets do not automatically bind them to CSS properties. Presets must provide a `css: ({ dt }) => ...` hook on the component in `orbitTheme.ts` to emit the CSS declarations.

---

## 7. Tailwind `backdrop-blur-(--var)` Nests `blur()`
- Never use Tailwind's `backdrop-blur-(--ui-overlay-blur)` arbitrary property utility. Tailwind compiles this to `backdrop-filter: blur(var(--ui-overlay-blur))`, resulting in nested `blur(blur(20px))` which is invalid CSS at runtime. Use explicit CSS declarations or the preset `css` hook referencing the token directly.
