import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), '')
  
  const apiUrl = env.VITE_API_URL || 'http://localhost:3000'
  const apiKey = env.VITE_API_KEY
  const apiKeyFormat = env.VITE_API_KEY_FORMAT || 'x-api-key'

  return {
    plugins: [react()],
    server: {
      proxy: {
        '/evaluate': {
          target: apiUrl,
          changeOrigin: true,
          configure: (proxy, _options) => {
            // Forward API key header from environment if available
            // Supports both x-api-key and Authorization: Bearer formats
            if (apiKey) {
              proxy.on('proxyReq', (proxyReq, req, _res) => {
                if (apiKeyFormat === 'bearer') {
                  proxyReq.setHeader('Authorization', `Bearer ${apiKey}`);
                } else {
                  proxyReq.setHeader('x-api-key', apiKey);
                }
              });
            }
          },
        },
        '/rules': {
          target: apiUrl,
          changeOrigin: true,
          configure: (proxy, _options) => {
            // Forward API key header from environment if available
            // Supports both x-api-key and Authorization: Bearer formats
            if (apiKey) {
              proxy.on('proxyReq', (proxyReq, req, _res) => {
                if (apiKeyFormat === 'bearer') {
                  proxyReq.setHeader('Authorization', `Bearer ${apiKey}`);
                } else {
                  proxyReq.setHeader('x-api-key', apiKey);
                }
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
