/// <reference types="vitest" />
import { defineConfig, loadEnv } from 'vite';
import type { UserConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vite.dev/config/
export default defineConfig(({ mode }): UserConfig => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), '');
  
  const apiUrl = env.VITE_API_URL || 'http://localhost:3000';
  const apiKey = env.VITE_API_KEY;
  const apiKeyFormat = env.VITE_API_KEY_FORMAT || 'x-api-key';

  const configureProxy = (proxy: any) => {
    // Forward API key header from environment if available
    // Supports both x-api-key and Authorization: Bearer formats
    if (apiKey) {
      proxy.on('proxyReq', (proxyReq: any, _req: any, _res: any) => {
        if (apiKeyFormat === 'bearer') {
          proxyReq.setHeader('Authorization', `Bearer ${apiKey}`);
        } else {
          proxyReq.setHeader('x-api-key', apiKey);
        }
      });
    }
  };

  return {
    plugins: [react()],
    server: {
      proxy: {
        '/api': {
          target: apiUrl,
          changeOrigin: true,
          rewrite: (path: string) => path.replace(/^\/api/, ''),
          configure: configureProxy,
        },
        '/evaluate': {
          target: apiUrl,
          changeOrigin: true,
          configure: configureProxy,
        },
        '/rules': {
          target: apiUrl,
          changeOrigin: true,
          configure: configureProxy,
        },
        '/health': {
          target: apiUrl,
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
});
