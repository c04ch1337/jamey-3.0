import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), '')
  
  const apiUrl = env.VITE_API_URL || 'http://localhost:3000'
  const apiKey = env.VITE_API_KEY

  return {
    plugins: [react()],
    server: {
      proxy: {
        '/evaluate': {
          target: apiUrl,
          changeOrigin: true,
          configure: (proxy, _options) => {
            // Forward API key header from environment if available
            if (apiKey) {
              proxy.on('proxyReq', (proxyReq, req, _res) => {
                proxyReq.setHeader('x-api-key', apiKey);
              });
            }
          },
        },
        '/rules': {
          target: apiUrl,
          changeOrigin: true,
          configure: (proxy, _options) => {
            // Forward API key header from environment if available
            if (apiKey) {
              proxy.on('proxyReq', (proxyReq, req, _res) => {
                proxyReq.setHeader('x-api-key', apiKey);
              });
            }
          },
        },
        '/health': {
          target: apiUrl,
          changeOrigin: true,
        },
      },
    },
  }
})
