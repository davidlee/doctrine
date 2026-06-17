import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: 'index.html',
    },
  },
  publicDir: 'public',
});
