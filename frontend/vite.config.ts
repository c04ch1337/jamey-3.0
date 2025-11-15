import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Get API URL from environment variable
const API_URL = process.env.VITE_API_URL || 'http://localhost:3000';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Proxy all /api requests to backend
      '/api': {
        target: API_URL,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      // Also proxy specific routes for backward compatibility
      '/evaluate': {
        target: API_URL,
        changeOrigin: true,
      },
      '/rules': {
        target: API_URL,
        changeOrigin: true,
      },
    },
  },
  build: {
    // Production build optimizations
    minify: 'esbuild',
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'query-vendor': ['@tanstack/react-query'],
          'axios-vendor': ['axios'],
        },
      },
    },
  },
})
