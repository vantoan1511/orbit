import './assets/main.css'

import 'primeicons/primeicons.css'

import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import { createApp } from 'vue'
import { init } from './services/nativeService'

import App from './App.vue'
import { initAppearance } from './composables/useAppearance'
import router from './router'
import { useNotificationStore } from './stores/notificationStore'
import { useShortcutStore } from './stores/shortcutStore'
import { useTableFilterStore } from './stores/tableFilterStore'
import { useTabsStore } from './stores/tabsStore'
import { Noir } from './theme/orbitTheme'

import ConfirmationService from 'primevue/confirmationservice'
import DialogService from 'primevue/dialogservice'
import ToastService from 'primevue/toastservice'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(PrimeVue, {
  ripple: true,
  theme: {
    preset: Noir,
    options: {
      darkModeSelector: '.my-app-dark',
      cssLayer: {
        name: 'primevue',
        order: 'theme, base, primevue, utilities'
      }
    }
  },
  pt: {
    dialog: {
      root: {
        class:
          'bg-(--ui-dialog-bg) border border-(--border) shadow-[var(--ui-card-shadow,var(--shadow))] rounded-[var(--ui-radius-card)] overflow-hidden'
      },
      mask: {
        class: 'bg-(--ui-mask-bg) backdrop-blur-(--ui-mask-blur) transition-all duration-200'
      },
      header: {
        class:
          'border-b border-(--border)/60 px-5 py-3.5 bg-transparent text-primary flex items-center justify-between'
      },
      content: {
        class: 'p-5 bg-transparent text-primary'
      },
      footer: {
        class:
          'border-t border-(--border)/60 px-5 py-3 bg-transparent flex items-center justify-end gap-2'
      },
      closeButton: {
        class:
          'w-7 h-7 p-0 text-muted-color hover:text-primary hover:bg-(--bg-hover) rounded-[var(--ui-radius-control)] focus:outline-none focus-visible:ring-1 focus-visible:ring-primary'
      }
    }
  }
})
app.use(ToastService)
app.use(ConfirmationService)
app.use(DialogService)

app.mount('#app')

init()

// Hydrate persistent state asynchronously once the app is mounted and Tauri host is initialized
void Promise.allSettled([
  initAppearance(),
  useShortcutStore(pinia).init(),
  useTableFilterStore(pinia).init(),
  useNotificationStore(pinia).init(),
  useTabsStore(pinia).init()
]).then((results) => {
  results.forEach((result) => {
    if (result.status === 'rejected') {
      console.warn('Store hydration encountered an error:', result.reason)
    }
  })
})
