import './assets/main.css'

import 'primeicons/primeicons.css'

import { init } from './services/nativeService'
import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { Noir } from './theme/orbitTheme'

import ToastService from 'primevue/toastservice'
import ConfirmationService from 'primevue/confirmationservice'
import DialogService from 'primevue/dialogservice'

const app = createApp(App)

app.use(createPinia())
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
  },
  pt: {
    dialog: {
      root: {
        class: 'bg-(--bg-card) border border-(--border) shadow-xl rounded-lg overflow-hidden'
      },
      header: { class: 'p-4 border-b border-(--border)' },
      content: { class: 'p-4' },
      footer: { class: 'p-4 border-t border-(--border) flex justify-end gap-2' }
    },
    dynamicdialog: {
      root: {
        class: 'bg-(--bg-card) border border-(--border) shadow-xl rounded-lg overflow-hidden'
      },
      header: { class: 'p-4 border-b border-(--border)' },
      content: { class: 'p-4' },
      footer: { class: 'p-4 border-t border-(--border) flex justify-end gap-2' }
    },
    confirmdialog: {
      root: {
        class: 'bg-(--bg-card) border border-(--border) shadow-xl rounded-lg overflow-hidden'
      },
      header: { class: 'p-4 border-b border-(--border)' },
      content: { class: 'p-4' },
      footer: { class: 'p-4 border-t border-(--border) flex justify-end gap-2' }
    }
  }
})
app.use(ToastService)
app.use(ConfirmationService)
app.use(DialogService)

app.mount('#app')

if (window.NL_PORT) {
  init()
}
