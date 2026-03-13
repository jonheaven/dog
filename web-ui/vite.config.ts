import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: path.resolve(__dirname, '../static/explorer'),
    emptyOutDir: true,
    sourcemap: false,
    rollupOptions: {
      input: path.resolve(__dirname, 'src/explorer/main.tsx'),
      output: {
        entryFileNames: 'explorer.js',
        chunkFileNames: 'chunks/[name]-[hash].js',
        assetFileNames: ({ name }) =>
          name?.endsWith('.css') ? 'explorer.css' : 'assets/[name]-[hash][extname]'
      }
    }
  }
});
