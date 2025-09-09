import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
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
