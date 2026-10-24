# Nuxt 4 Setup & Global Configuration

This reference covers the initial setup of PrimeVue in a Nuxt 4 environment and managing global behaviors.

## ⚙️ Nuxt 4 Module & Global Configuration

Declare configurations within the `primevue` block of `nuxt.config.ts` to manage global behavior.

### 1. Module Options Configuration
Use `options` to set global defaults like input variants, ripples, and basic theme options:

```typescript
import Aura from '@primeuix/themes/aura'

export default defineNuxtConfig({
  modules: ['@nuxt/eslint', '@primevue/nuxt-module'],
  primevue: {
    options: {
      ripple: true,            // Enable ripple animations globally
      inputVariant: 'filled',  // Outlined (default) or Filled (adds bg color to inputs)
      theme: {
        preset: Aura,
        options: {
          prefix: 'p',
          darkModeSelector: '.dark', // Match Tailwind CSS dark mode class
          cssLayer: {
            name: 'primevue',
            order: 'tailwind-base, primevue, tailwind-utilities'
          }
        }
      },
      zIndex: {
        modal: 1100,      // dialog, drawer
        overlay: 1000,    // select, popover, datepicker
        menu: 1000,       // overlay menus
        tooltip: 1100     // tooltips
      }
    }
  }
})
```

---

## 🌗 Dark Mode Coordination

To keep PrimeVue dark themes synced with Tailwind's dark utility class, configure the `darkModeSelector` to match the exact class (e.g. `.dark`) in your project config:

```typescript
// nuxt.config.ts
primevue: {
  options: {
    theme: {
      options: {
        darkModeSelector: '.dark'
      }
    }
  }
}
```

Then toggling dark mode is as simple as managing the class at the document root level:

```typescript
const toggleDarkMode = () => {
  const isDark = document.documentElement.classList.toggle('dark')
  localStorage.setItem('theme', isDark ? 'dark' : 'light')
}
```
