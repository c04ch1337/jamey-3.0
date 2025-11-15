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
