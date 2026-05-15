import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import { i18n } from './i18n'
import { checkForUpdate } from '@/composables/useUpdater'
import './assets/index.css'
import 'vue-sonner/style.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(i18n)
app.mount('#app')

// Non-blocking background update check on startup (R-M7.5)
checkForUpdate().catch(() => {})
