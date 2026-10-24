# Theming, Tailwind v4 & Pass-Through

This reference defines how to customize the visual language of PrimeVue components using design tokens, Tailwind CSS v4, and Pass-Through attributes.

## 🎨 Design Tokens & Custom Presets

The theme is built on **Primitive**, **Semantic**, and **Component** design tokens.

### 1. Defining a Custom Preset
Use `definePreset` from `@primeuix/themes` to customize an existing preset (e.g. Aura). Always check the original preset structure (colorScheme vs direct value) so customizations apply correctly to both light and dark modes.

```typescript
import { definePreset } from '@primeuix/themes'
import Aura from '@primeuix/themes/aura'

export const MyCustomPreset = definePreset(Aura, {
  semantic: {
    primary: {
      50: '{indigo.50}',
      100: '{indigo.100}',
      // ... (rest of tokens)
      950: '{indigo.950}'
    },
    focusRing: {
      width: '2px',
      style: 'dashed',
      color: '{primary.color}',
      offset: '1px'
    },
    colorScheme: {
      light: { formField: { hoverBorderColor: '{primary.color}' } },
      dark: { formField: { hoverBorderColor: '{primary.color}' } }
    }
  }
})
```

### 2. Runtime Dynamic Theming Utilities
```typescript
import { $dt, updatePrimaryPalette, updateSurfacePalette, usePreset } from '@primeuix/themes'

// Fetch token CSS variable
const primaryColorVar = $dt('primary.color').variable // var(--p-primary-color)

// Update the primary color palette dynamically
updatePrimaryPalette({ 50: '{violet.50}', 950: '{violet.950}' })

// Swap preset entirely
usePreset(newPreset)
```

---

## 🎨 Tailwind CSS v4 Integration & Extends

The Tailwind CSS plugin `@plugin "tailwindcss-primeui"` maps design tokens directly into Tailwind utility classes.

### 1. Semantic Tailwind Utilities
Use these specialized classes to build cohesive interfaces that automatically update with theme tokens:

- **Colors**: `primary`, `primary-contrast`, `primary-emphasis`, `primary-[50-950]`
- **Surfaces**: `surface-[0-950]`
- **Borders**: `border-surface` (matches content border color)
- **Backgrounds**: `bg-emphasis`, `bg-highlight`
- **Borders & Text**: `rounded-border`, `text-color`, `text-muted-color`

### 2. CSS Specificity Layers
Avoid using `!` (important) overrides. Group Tailwind styles into cascade layers in `assets/css/main.css`:

```css
@import 'primeicons/primeicons.css';
@plugin "tailwindcss-primeui";

@layer tailwind-base, primevue, tailwind-utilities;

@layer tailwind-base {
  @import 'tailwindcss/theme.css';
  @import 'tailwindcss/preflight.css';
}

@layer tailwind-utilities {
  @import 'tailwindcss/utilities.css';
}
```

---

## 🔌 Unstyled Mode & Pass-Through (`pt`)

When building highly customized designs without the default design tokens, use unstyled mode or targeted PT overrides.

### 1. Inline Pass-Through Style Definition
Inject Tailwind styling on targeted inner component classes using `pt:sectionName`:

```vue
<template>
  <Button
    label="Search"
    unstyled
    pt:root="bg-teal-600 hover:bg-teal-700 py-2 px-4 rounded-full border-0 flex gap-2 items-center"
    pt:label="text-white font-bold"
  />
</template>
```

### 2. Global Pass-Through Configuration
```typescript
primevue: {
  options: {
    unstyled: true,
    pt: {
      button: {
        root: 'bg-teal-600 px-4 py-2 rounded-xl border-0 flex gap-2',
        label: 'text-white'
      }
    }
  }
}
```
