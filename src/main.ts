import './assets/main.css'

import 'primeicons/primeicons.css'

import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import { createApp } from 'vue'
import { init } from './services/nativeService'

import App from './App.vue'
import { initTheme } from './composables/useTheme'
import router from './router'
import { useNotificationStore } from './stores/notificationStore'
import { useTableFilterStore } from './stores/tableFilterStore'
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
        order: 'theme, base, primevue'
      }
    }
  }
})
app.use(ToastService)
app.use(ConfirmationService)
app.use(DialogService)

app.mount('#app')

if (typeof window !== 'undefined' && window.NL_PORT) {
  init()
}

// Hydrate persistent state asynchronously once the app is mounted and Neutralino is initialized
void Promise.allSettled([
  initTheme(),
  useTableFilterStore(pinia).init(),
  useNotificationStore(pinia).init()
]).then((results) => {
  results.forEach((result) => {
    if (result.status === 'rejected') {
      console.warn('Store hydration encountered an error:', result.reason)
    }
  })
})
