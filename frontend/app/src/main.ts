import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import App from './App.vue'
import { router } from './app/router'
import './styles.css'
import { initializeTheme } from './app/theme'

initializeTheme()

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(PiniaColada, { pinia })
app.use(router)
app.mount('#app')
