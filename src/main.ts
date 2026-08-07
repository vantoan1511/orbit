import './assets/main.css'

import 'primeicons/primeicons.css'

import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import { createApp } from 'vue'
import { init } from './services/nativeService'

import App from './App.vue'
import router from './router'
import { Noir } from './theme/orbitTheme'

import ConfirmationService from 'primevue/confirmationservice'
import DialogService from 'primevue/dialogservice'
import ToastService from 'primevue/toastservice'

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
  }
})
app.use(ToastService)
app.use(ConfirmationService)
app.use(DialogService)

app.mount('#app')

if (window.NL_PORT) {
  init()
}
