import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

const apiTarget = 'http://api:8080'

export default defineConfig({
  plugins: [vue()],
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
    allowedHosts: ['localhost'],
    hmr: {
      host: 'localhost',
      clientPort: 4173,
      protocol: 'ws',
    },
    proxy: {
      '/api': {
        target: apiTarget,
        changeOrigin: false,
      },
      '/health': {
        target: apiTarget,
        changeOrigin: false,
      },
    },
  },
})
