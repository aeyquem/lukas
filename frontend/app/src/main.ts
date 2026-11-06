import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import App from './app.vue'
import { router } from './app/router'
import './styles.css'
import { useAppearanceStore } from './app/theme-store'

const app = createApp(App)
const pinia = createPinia()

useAppearanceStore(pinia).initializeTheme()

app.use(pinia)
app.use(PiniaColada, { pinia })
app.use(router)
app.mount('#app')
