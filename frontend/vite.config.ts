<<<<<<< HEAD
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
=======
/// <reference types="vitest" />
import { defineConfig } from 'vitest/config';
import type { UserConfig } from 'vite';
import react from '@vitejs/plugin-react';

const API_URL = process.env.VITE_API_URL || 'http://localhost:3000';

const viteConfig: UserConfig = {
  plugins: [react()],
  server: {
    proxy: {
      '/api': {
        target: API_URL,
        changeOrigin: true,
        rewrite: (path: string) => path.replace(/^\/api/, ''),
      },
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
    minify: 'esbuild',
    sourcemap: false,
    target: 'es2020',
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'query-vendor': ['@tanstack/react-query'],
          'axios-vendor': ['axios'],
          'zod-vendor': ['zod'],
        },
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]',
      },
    },
    chunkSizeWarningLimit: 1000,
    cssCodeSplit: true,
  },
  optimizeDeps: {
    include: ['react', 'react-dom', '@tanstack/react-query', 'axios', 'zod'],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/setupTests.ts',
    css: true,
  },
};

export default defineConfig(viteConfig);
>>>>>>> origin/main
