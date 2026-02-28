import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      thresholds: {
        statements: 100,
        branches: 100,
        functions: 100,
        lines: 100
      }
    }
  },
  server: {
    port: 5173
  },
  // Ensure workers and code-split outputs use ES modules (avoid IIFE/UMD formats)
  worker: {
    format: 'es'
  },
  build: {
    rollupOptions: {
      output: {
        format: 'es'
      }
    }
  }
});
